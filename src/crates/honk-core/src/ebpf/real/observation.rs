use super::*;
use crate::ebpf::interface::{Binding, Direction};
use crate::ebpf::{
    DatapathAttachment, DatapathCheck, DatapathKind, DatapathObservation, DatapathObservationError,
    DatapathProgram, MAX_DATAPATH_ATTACHMENTS, MAX_DATAPATH_PROGRAMS,
};
use aya::maps::IterableMap;
use aya::programs::{ProgramError, links::FdLink};

impl RealEbpfBackend {
    pub(super) fn observe_backend(&self) -> DatapathObservation {
        let mut result = DatapathObservation::unknown(DatapathKind::Real);
        let Some(loader) = &self.programs else {
            result.programs = DatapathCheck::Absent;
            result.hooks = DatapathCheck::Absent;
            result.routing = DatapathCheck::Absent;
            return result;
        };
        result.listeners_published = Some(self.listeners_published);
        if let Some(bpf) = loader.try_object() {
            result.programs = DatapathCheck::Absent;
            for (index, (name, program)) in bpf.programs().enumerate() {
                if index == MAX_DATAPATH_PROGRAMS {
                    result.programs = DatapathCheck::Unknown;
                    result.record_error(DatapathObservationError::Limit);
                    break;
                }
                match program.info() {
                    Ok(info) => {
                        result.loaded_programs.push(DatapathProgram {
                            name: name.to_owned(),
                            id: info.id(),
                        });
                        if result.programs != DatapathCheck::Error {
                            result.programs = DatapathCheck::Verified;
                        }
                    }
                    Err(ProgramError::NotLoaded) => {}
                    Err(_) => {
                        result.programs = DatapathCheck::Error;
                        result.record_error(DatapathObservationError::Programs);
                    }
                }
            }
        } else {
            result.programs = DatapathCheck::Unknown;
        }
        match self.observed_routing_generation() {
            Ok(Some(generation)) => {
                result.routing = DatapathCheck::Verified;
                result.routing_generation = Some(generation);
            }
            Ok(None) => result.routing = DatapathCheck::Absent,
            Err(_) => {
                result.routing = DatapathCheck::Error;
                result.record_error(DatapathObservationError::Routing);
            }
        }
        match self.array_get::<u32>("DATAPATH_STATE_MAP", 0) {
            Ok(Some(0)) => result.admission = Some(false),
            Ok(Some(1)) => result.admission = Some(true),
            _ => result.record_error(DatapathObservationError::Admission),
        }
        match self
            .hash_map::<TuplesKey, FlowMapValue<ConnState>>("CONN_STATE_MAP")
            .and_then(|map| Ok(map.map().info()?.max_entries()))
        {
            Ok(capacity) => result.conn_state_capacity = Some(capacity),
            Err(_) => result.record_error(DatapathObservationError::Maps),
        }
        // A retained link FD alone does not prove its hook is still installed.
        // TCX queries below check the host interfaces and dae0. aya exposes no
        // query for the cgroup and sk_lookup hooks, and dae0peer sits in daens,
        // so those required hooks stay unchecked and `hooks` stays unknown.
        let owned = self.bindings.owned();
        if owned.is_empty()
            && self.cgroup_sock_links.is_empty()
            && self.cgroup_sock_addr_links.is_empty()
            && self.dae0_ingress_link.is_none()
            && self.dae0peer_ingress_link.is_none()
            && self.sk_lookup_link.is_none()
        {
            result.hooks = DatapathCheck::Absent;
        }
        let tracked = owned.len() + usize::from(self.dae0_ingress_link.is_some());
        let dae0 = self
            .dae0_ingress_link
            .as_ref()
            .filter(|_| owned.len() < MAX_DATAPATH_ATTACHMENTS)
            .map(|link| observe_dae0(link, &result.loaded_programs));
        let bound = owned
            .iter()
            .take(MAX_DATAPATH_ATTACHMENTS)
            .map(|(binding, link_id, fd)| {
                observe_binding(binding, *link_id, fd.as_raw_fd(), &result.loaded_programs)
            })
            .chain(dae0)
            .collect::<Vec<_>>();
        for observed in bound {
            match observed {
                Ok(attachment) => {
                    if attachment.state != DatapathCheck::Verified {
                        result.record_error(DatapathObservationError::Hooks);
                    }
                    result.attachments.push(attachment);
                }
                Err(_) => result.record_error(DatapathObservationError::Hooks),
            }
        }
        if tracked > MAX_DATAPATH_ATTACHMENTS {
            result.record_error(DatapathObservationError::Limit);
        }
        // Every hook the real datapath owns is required: the configured TCX
        // hooks, the cgroup hooks when cgroup v2 is present, and dae0_ingress,
        // dae0peer_ingress and tproxy_sk_lookup, which startup always attaches.
        let verified_extra = self.verify_non_host_tc_hooks(&mut result);
        let required =
            owned.len() + self.cgroup_sock_links.len() + self.cgroup_sock_addr_links.len() + 3;
        result.verify_required_hooks(required.saturating_sub(verified_extra));
        result.checked_at = std::time::SystemTime::now();
        result
    }

    fn verify_non_host_tc_hooks(&self, result: &mut DatapathObservation) -> usize {
        use aya_obj::generated::bpf_attach_type;

        let mut verified = 0usize;
        if !self.cgroup_sock_links.is_empty() || !self.cgroup_sock_addr_links.is_empty() {
            let (create_name, cg_addr_names) = match self.pname_mode {
                process_name::PnameCaptureMode::Argv0 => (
                    "tproxy_wan_cg_sock_create",
                    [
                        "tproxy_wan_cg_connect4",
                        "tproxy_wan_cg_connect6",
                        "tproxy_wan_cg_sendmsg4",
                        "tproxy_wan_cg_sendmsg6",
                    ],
                ),
                process_name::PnameCaptureMode::Comm => (
                    "tproxy_wan_cg_sock_create_comm",
                    [
                        "tproxy_wan_cg_connect4_comm",
                        "tproxy_wan_cg_connect6_comm",
                        "tproxy_wan_cg_sendmsg4_comm",
                        "tproxy_wan_cg_sendmsg6_comm",
                    ],
                ),
            };
            match detect_cgroup_path().and_then(|p| Ok(std::fs::File::open(p)?)) {
                Ok(cgroup_file) => {
                    let fd = cgroup_file.as_raw_fd();
                    if !self.cgroup_sock_links.is_empty() {
                        match verify_target_prog(
                            fd,
                            bpf_attach_type::BPF_CGROUP_INET_SOCK_CREATE as u32,
                            create_name,
                            &result.loaded_programs,
                        ) {
                            Ok(true) => verified += self.cgroup_sock_links.len(),
                            _ => result.record_error(DatapathObservationError::Hooks),
                        }
                    }
                    if self.cgroup_sock_addr_links.len() == cg_addr_names.len() {
                        let types = [
                            bpf_attach_type::BPF_CGROUP_INET4_CONNECT as u32,
                            bpf_attach_type::BPF_CGROUP_INET6_CONNECT as u32,
                            bpf_attach_type::BPF_CGROUP_UDP4_SENDMSG as u32,
                            bpf_attach_type::BPF_CGROUP_UDP6_SENDMSG as u32,
                        ];
                        for (name, attach_type) in cg_addr_names.into_iter().zip(types) {
                            match verify_target_prog(fd, attach_type, name, &result.loaded_programs)
                            {
                                Ok(true) => verified += 1,
                                _ => result.record_error(DatapathObservationError::Hooks),
                            }
                        }
                    } else if !self.cgroup_sock_addr_links.is_empty() {
                        result.record_error(DatapathObservationError::Hooks);
                    }
                }
                Err(_) => result.record_error(DatapathObservationError::Hooks),
            }
        }

        if let Some(link) = &self.dae0peer_ingress_link {
            let progs = &result.loaded_programs;
            match crate::daens::with_daens_netns("observe dae0peer_ingress", || {
                observe_named_tc(link, "dae0peer", progs)
            }) {
                Ok(attachment) if attachment.state == DatapathCheck::Verified => verified += 1,
                _ => result.record_error(DatapathObservationError::Hooks),
            }
        }

        if self.sk_lookup_link.is_some() {
            match crate::daens::daens_fd().and_then(|netns| {
                verify_target_prog(
                    netns.as_raw_fd(),
                    bpf_attach_type::BPF_SK_LOOKUP as u32,
                    "tproxy_sk_lookup",
                    &result.loaded_programs,
                )
            }) {
                Ok(true) => verified += 1,
                _ => result.record_error(DatapathObservationError::Hooks),
            }
        }

        verified
    }
}

fn verify_target_prog(
    target_fd: RawFd,
    attach_type: u32,
    prog_name: &str,
    programs: &[DatapathProgram],
) -> anyhow::Result<bool> {
    let expected = programs
        .iter()
        .find(|p| p.name == prog_name)
        .ok_or_else(|| anyhow::anyhow!("program {prog_name} not loaded"))?
        .id;
    let attached = bindings::query_target_progs(target_fd, attach_type)?;
    Ok(attached.contains(&expected))
}

/// An owned interface binding: the held link must be the one the owner
/// recorded, on the same interface index, and on that interface's chain.
fn observe_binding(
    binding: &Binding,
    link_id: u32,
    link: RawFd,
    programs: &[DatapathProgram],
) -> anyhow::Result<DatapathAttachment> {
    let info = bindings::link_info(link)?;
    anyhow::ensure!(
        info.id == link_id
            && info.ifindex == binding.ifindex.get()
            && info.attach_type == bindings::tcx_attach_type(binding.direction),
        "interface identity changed during observation"
    );
    let program = programs
        .iter()
        .find(|program| program.id == info.program_id)
        .ok_or_else(|| anyhow::anyhow!("hook program is not owned"))?;
    let verified = bindings::query_chain(binding.ifindex.get(), binding.direction)
        .map_err(|error| anyhow::anyhow!("query TCX chain: {error}"))?
        .contains_link(link_id);
    Ok(DatapathAttachment {
        program: program.name.clone(),
        interface: binding.interface.clone(),
        egress: binding.direction == Direction::Egress,
        state: if verified {
            DatapathCheck::Verified
        } else {
            DatapathCheck::Absent
        },
    })
}

/// The dae0 ingress link is an Aya link: its IDs come from Aya, the chain from
/// the same private query.
fn observe_dae0(
    link: &aya::programs::tc::SchedClassifierLink,
    programs: &[DatapathProgram],
) -> anyhow::Result<DatapathAttachment> {
    observe_named_tc(link, "dae0", programs)
}

fn observe_named_tc(
    link: &aya::programs::tc::SchedClassifierLink,
    ifname: &str,
    programs: &[DatapathProgram],
) -> anyhow::Result<DatapathAttachment> {
    let ifindex = nix::net::if_::if_nametoindex(ifname)?;
    let fd: &FdLink = link.try_into()?;
    let info = fd.info()?;
    let program = programs
        .iter()
        .find(|program| program.id == info.program_id())
        .ok_or_else(|| anyhow::anyhow!("hook program is not owned"))?;
    let verified = bindings::query_chain(ifindex, Direction::Ingress)
        .map_err(|error| anyhow::anyhow!("query TCX chain: {error}"))?
        .contains_link(info.id());
    Ok(DatapathAttachment {
        program: program.name.clone(),
        interface: ifname.to_owned(),
        egress: false,
        state: if verified {
            DatapathCheck::Verified
        } else {
            DatapathCheck::Absent
        },
    })
}

// Loads the wasm-bindgen (nodejs target) output built by `rust-anything build`.
const native = require('./pkg/dae_rs.js')

// Structured values cross as JSON text. JSON has no `undefined`, NaN, -0, bigint, Date, Map, Set
// or bytes, and Rust strings no lone surrogates: those travel as one-key marker objects
// (`{"$undefined": true}`, `{"$num": "NaN"}`, `{"$map": [[k, v]]}`, `{"$bytes": [...]}`, ...),
// the same markers recordings use.
const replacer = function (k, v) {
  const raw = this[k]
  if (raw instanceof Date) return { $date: Number.isNaN(raw.getTime()) ? 'Invalid' : raw.toISOString() }
  if (typeof Buffer !== 'undefined' && Buffer.isBuffer(raw)) return { $buffer: Array.from(raw) }
  if (raw instanceof Uint8Array) return { $bytes: Array.from(raw) }
  if (raw instanceof Map) return { $map: [...raw] }
  if (raw instanceof Set) return { $set: [...raw] }
  return v === undefined ? { $undefined: true }
    : typeof v === 'bigint' ? { $bigint: String(v) }
    : typeof v === 'number' ? (Number.isFinite(v) && !Object.is(v, -0) ? v : { $num: Object.is(v, -0) ? '-0' : String(v) })
    : typeof v === 'string' && !v.isWellFormed() ? { $utf16: Array.from({ length: v.length }, (_, i) => v.charCodeAt(i)) }
    : v
}
const encode = v => JSON.stringify(v, replacer)
const single = (v, key) => Object.keys(v).length === 1 && key in v
const revive = v => Array.isArray(v) ? v.map(revive)
  : v !== null && typeof v === 'object'
    ? (single(v, '$undefined') ? undefined
      : single(v, '$utf16') ? fromUnits(v.$utf16)
      : single(v, '$num') ? Number(v.$num)
      : single(v, '$bigint') ? BigInt(v.$bigint)
      : single(v, '$date') ? new Date(v.$date)
      : single(v, '$bytes') ? Uint8Array.from(v.$bytes)
      : single(v, '$buffer') ? Buffer.from(v.$buffer)
      : single(v, '$map') ? new Map(v.$map.map(([k, x]) => [revive(k), revive(x)]))
      : single(v, '$set') ? new Set(v.$set.map(revive))
      : Object.fromEntries(Object.entries(v).map(([k, x]) => [k, revive(x)])))
    : v
// Most results carry no marker: skip the walk unless the text has one.
const decode = text => {
  const value = JSON.parse(text)
  return text.includes('"$') ? revive(value) : value
}
const fromUnits = (units) => {
  let out = ''
  for (let i = 0; i < units.length; i += 8192)
    out += String.fromCharCode.apply(null, Array.prototype.slice.call(units, i, i + 8192))
  return out
}

module.exports = { ...native }
const specs = {
  serializeConditions: [[1], 'raw', false],
  parseConditions: [[0, 0], 'json', false],
  flagKey: [[0], 'json', false],
  validFlagName: [[0], 'json', false],
  hasEmbeddedFlag: [[0], 'json', false],
  validFlagChoice: [[1], 'raw', false],
  flagChoices: [[0], 'json', false],
  matchPreset: [[1], 'json', false],
  missingCategories: [[1, 1], 'json', false],
  geodataPreset: [[0], 'json', false],
  conditionFamily: [[1], 'json', false],
  newGroupCondition: [[], 'json', false],
  conditionValues: [[0], 'json', false],
  parseConditionValues: [[0], 'json', false],
  parseGroupConditions: [[0], 'json', false],
  serializeGroupConditions: [[1], 'raw', false],
  groupFilterDraft: [[0, 0], 'json', false],
  groupFilterText: [[1], 'json', false],
  groupFilterTexts: [[1], 'json', false],
  reconcileGroupFilters: [[1, 0], 'json', false],
  newGroupFilter: [[], 'json', false],
  groupAdmits: [[0, 1], 'json', false],
  readGroupEntries: [[0], 'json', false],
  quoteName: [[0], 'json', false],
  isWritableName: [[0], 'json', false],
  groupNameProblem: [[0, 1], 'json', false],
  writeGroupEntry: [[0, 0, 1], 'raw', false],
  nameText: [[1, 1], 'json', false],
  nestedIn: [[1], 'raw', false],
  exactTokens: [[0, 1], 'json', false],
  namedIn: [[1], 'raw', false],
  classifyFilters: [[1], 'json', false],
  describeFilters: [[0], 'json', false],
  citingGroups: [[0, 0], 'json', false],
  namedInExpression: [[0, 0], 'json', false],
  groupsNamingTag: [[0, 0], 'json', false],
  addSubtagsToGroup: [[0, 0, 0], 'json', false],
  addNodesToGroup: [[0, 0, 0], 'json', false],
  removeSubtagsFromGroup: [[0, 0, 0], 'json', false],
  ruleCondition: [[1, 0], 'raw', false],
  isGlob: [[0], 'json', false],
  includedBy: [[0, 0], 'json', false],
  resolveIncludePath: [[0, 0], 'json', false],
  newSourcePathProblem: [[0], 'json', false],
  namePatterns: [[0], 'json', false],
  newSourceNameProblem: [[0, 1], 'json', false],
  sourceAt: [[1, 0], 'json', false],
  protocolName: [[1], 'json', false],
  emptyNode: [[1], 'json', false],
  parseNodeLink: [[0], 'json', false],
  nodeLink: [[1, 1], 'raw', false],
  securityOptions: [[1], 'json', false],
  usesTls: [[1], 'json', false],
  manualFields: [[1], 'json', false],
  advancedFields: [[1], 'json', false],
  settle: [[1], 'json', false],
  manualInvalid: [[1], 'raw', false],
  manualLink: [[1], 'raw', false],
  isNodeLink: [[0], 'json', false],
  readNodeEntries: [[0], 'json', false],
  readChainEntries: [[0], 'json', false],
  nodeReferenced: [[0, 0], 'json', false],
  nodeReferrers: [[0, 0], 'json', false],
  removeChainEntry: [[0, 1], 'raw', false],
  chainHopCandidate: [[1], 'json', false],
  tunnelDialable: [[1], 'json', false],
  chainHopProblem: [[1, 0, 0], 'raw', false],
  chainUdpProblem: [[1], 'json', false],
  writeChainEntry: [[0, 1, 1], 'raw', false],
  appendChainEntry: [[0, 1], 'raw', false],
  writeNodeEntry: [[0, 1, 1], 'raw', false],
  readMode: [[0], 'json', false],
  writeMode: [[0, 1], 'raw', false],
  sameMode: [[1, 1], 'json', false],
  modeRuleLines: [[0], 'json', false],
  nodeProbeSupport: [[0], 'json', false],
  supportsNodeProbe: [[0, 1, 1], 'raw', false],
  regionFlag: [[0], 'json', false],
  matchRegion: [[0], 'json', false],
  sourceFor: [[1, 1], 'json', false],
  ruleAnchor: [[1, 1, 1], 'json', false],
  dnsRuleTarget: [[1], 'json', false],
  dnsRuleAnchor: [[1, 1, 0, 1], 'json', false],
  dnsListEnd: [[1, 0], 'json', false],
  dnsUpstreams: [[0, 1], 'json', false],
  dnsUpstreamNames: [[0, 1], 'raw', false],
  ruleLine: [[0, 0, 0], 'json', false],
  addRule: [[0, 1, 0, 0, 0], 'raw', false],
  replaceRuleTarget: [[0, 1, 0, 0], 'raw', false],
  removeRule: [[0, 1], 'raw', false],
  replaceRule: [[0, 1, 0, 0, 0], 'raw', false],
  addFallback: [[0, 1, 0], 'raw', false],
  numberSetting: [[1], 'raw', false],
  settingValue: [[1, 0], 'raw', false],
  serializeSetting: [[1, 0], 'raw', false],
  writeSettings: [[0, 1, 0, 1], 'raw', false],
  allGroupNames: [[1, 1], 'json', false],
  allUpstreamNames: [[1, 1], 'json', false],
  redacted: [[1], 'json', false],
  fileName: [[1], 'json', false],
  validationSources: [[1, 1], 'json', false],
  restartSettings: [[1], 'json', false],
  restartRequired: [[1], 'json', false],
  unwrittenRestartSettings: [[1, 1], 'json', false],
  completeSubscriptionUrl: [[0], 'json', false],
  isSubscriptionUrl: [[0], 'json', false],
  parseInterval: [[0], 'json', false],
  urlHost: [[0], 'json', false],
  agentProblem: [[0, 0], 'json', false],
  readSubscriptionEntries: [[0], 'json', false],
  writeSubscriptionEntry: [[0, 0, 1], 'raw', false],
  templateRules: [[0, 1], 'raw', false],
  detectTemplate: [[0], 'json', false],
  routingOptions: [[0], 'json', false],
  isBareName: [[0], 'json', false],
  scanConfig: [[0], 'json', false],
  unquote: [[0], 'json', false],
  blockFields: [[0, 1, 1], 'json', false],
  blockBody: [[0, 1], 'raw', false],
  blockEntries: [[0, 1], 'json', false],
  policyKind: [[0], 'json', false],
  readXhttp: [[0], 'json', false],
  duplicateXhttpHeaders: [[1], 'json', false],
  writeXhttp: [[0, 1], 'raw', false],
}
const classes = {

}
const convert = result => result === 'json' ? decode : result === 'units' ? fromUnits : x => x
// An omitted structured argument is `undefined`, not a missing value.
const prepare = (a, structured) => Array.from({ length: Math.max(a.length, structured.length) }, (_, i) => structured[i] ? encode(a[i]) : a[i])
// Functions: [structured arguments, result kind, async].
for (const [name, [structured, result, isAsync]] of Object.entries(specs)) {
  const call = native[name]
  const out = convert(result)
  module.exports[name] = isAsync
    ? async (...a) => out(await call(...prepare(a, structured)))
    : (...a) => out(call(...prepare(a, structured)))
}
// Classes: a JS class around the native one, converting arguments and results.
const INNER = Symbol('native')
for (const [name, [ctor, methods]] of Object.entries(classes)) {
  const Native = native[name]
  const Wrapped = { [name]: class { constructor(...a) { this[INNER] = new Native(...prepare(a, ctor)) } } }[name]
  for (const [method, [structured, result]] of Object.entries(methods)) {
    const out = convert(result)
    Wrapped.prototype[method] = function (...a) { return out(this[INNER][method](...prepare(a, structured))) }
  }
  module.exports[name] = Wrapped
}

// The binary wire (see rust-anything's `wire.rs`): arguments are written into `wIn` and copied into
// wasm memory in one step; results are read where the Rust side left them.
let wIn = new Uint8Array(1 << 16), iv = new DataView(wIn.buffer), at = 0, p = 0
const mem = native.rustAnythingWireMemory()
let wOut = new Uint8Array(mem.buffer), ov = new DataView(mem.buffer)
const regrow = () => {}
// Growing wasm memory replaces its buffer: views are made again when that happened.
const views = () => {
  if (wOut.buffer !== mem.buffer) {
    wOut = new Uint8Array(mem.buffer)
    ov = new DataView(mem.buffer)
  }
}
const wireCall = fn => {
  const ptr = native.rustAnythingWireIn(at)
  views()
  wOut.set(wIn.subarray(0, at), ptr)
  if (fn(at) === 0xFFFFFFFF) return false
  views()
  p = native.rustAnythingWireOut()
  return true
}

const need = n => {
  if (at + n <= wIn.length) return
  const grown = new Uint8Array(Math.max(wIn.length * 2, at + n))
  grown.set(wIn)
  wIn = grown
  iv = new DataView(wIn.buffer)
  regrow()
}
const wU8 = v => { need(1); iv.setUint8(at++, v) }
const wU32 = v => { need(4); iv.setUint32(at, v, true); at += 4 }
const wF64 = v => { need(8); iv.setFloat64(at, v, true); at += 8 }
const wUnits = s => {
  need(4 + 2 * s.length)
  iv.setUint32(at, s.length, true)
  at += 4
  for (let i = 0; i < s.length; i++) { iv.setUint16(at, s.charCodeAt(i), true); at += 2 }
}
const wStr = s => { wU8(4); wUnits(s) }
const wDyn = v => {
  switch (typeof v) {
    case 'undefined': wU8(0); return true
    case 'boolean': wU8(v ? 3 : 2); return true
    case 'number': wU8(4); wF64(v); return true
    case 'string': wU8(5); wUnits(v); return true
    case 'object':
      if (v === null) { wU8(1); return true }
      if (Array.isArray(v)) {
        wU8(6); wU32(v.length)
        for (let i = 0; i < v.length; i++) if (!wDyn(v[i])) return false
        return true
      }
      if (Object.getPrototypeOf(v) !== Object.prototype && Object.getPrototypeOf(v) !== null) return false
      {
        const keys = Object.keys(v)
        wU8(7); wU32(keys.length)
        for (const k of keys) { wUnits(k); if (!wDyn(v[k])) return false }
        return true
      }
  }
  return false
}
const rU8 = () => ov.getUint8(p++)
const rU32 = () => { const v = ov.getUint32(p, true); p += 4; return v }
const rF64 = () => { const v = ov.getFloat64(p, true); p += 8; return v }
// ignoreBOM: a leading U+FEFF is part of the string, not a byte-order mark.
const utf16 = new TextDecoder('utf-16le', { ignoreBOM: true })
const rUnits = (wellFormed) => {
  const n = rU32()
  let s
  if (n > 24 && wellFormed) s = utf16.decode(wOut.subarray(p, p + 2 * n))
  else {
    s = ''
    for (let i = 0; i < n; i++) s += String.fromCharCode(ov.getUint16(p + 2 * i, true))
  }
  p += 2 * n
  return s
}
// Tag 1: a string from Rust (well-formed); 4: raw code units, lone surrogates included.
const rStr = () => rUnits(rU8() === 1)
const rDyn = () => {
  switch (rU8()) {
    case 0: return undefined
    case 1: return null
    case 2: return false
    case 3: return true
    case 4: return rF64()
    case 5: return rUnits(false)
    case 6: { const n = rU32(); const a = new Array(n); for (let i = 0; i < n; i++) a[i] = rDyn(); return a }
    case 7: { const n = rU32(); const o = {}; for (let i = 0; i < n; i++) { const k = rUnits(false); o[k] = rDyn() } return o }
  }
}

// `serializeConditions`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.serializeConditions
const call = native.serializeConditionsWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.serializeConditions = function serializeConditions (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `parseConditions`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.parseConditions
const call = native.parseConditionsWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!Array.isArray(a1)) return false
wU8(1)
wU32(a1.length)
for (let i0 = 0; i0 < a1.length; i0++) {
const v0 = a1[i0]
if (!wDyn(v0)) return false
}
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
if (rU8() === 0) t = null
else {
const n1 = rU32()
const a1 = new Array(n1)
for (let i1 = 0; i1 < n1; i1++) {
let v1
{
const o2 = {}
const n2 = rU8()
for (let j2 = 0; j2 < n2; j2++) {
switch (rU8()) {
case 0: {
let v2
v2 = rDyn()
o2["kind"] = v2
break
}
case 1: {
let v2
v2 = rStr()
o2["value"] = v2
break
}
case 2: {
let v2
v2 = rU8() === 1
o2["negate"] = v2
break
}
default: {
const k2 = rStr()
o2[k2] = rDyn()
}
}
}
v1 = o2
}
a1[i1] = v1
}
t = a1
}
}
}
return t
}
module.exports.parseConditions = function parseConditions (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of parseConditions has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of parseConditions has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `flagKey`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.flagKey
const call = native.flagKeyWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.flagKey = function flagKey (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of flagKey has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `validFlagName`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.validFlagName
const call = native.validFlagNameWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.validFlagName = function validFlagName (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of validFlagName has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `hasEmbeddedFlag`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.hasEmbeddedFlag
const call = native.hasEmbeddedFlagWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.hasEmbeddedFlag = function hasEmbeddedFlag (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of hasEmbeddedFlag has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `validFlagChoice`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.validFlagChoice
const call = native.validFlagChoiceWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.validFlagChoice = function validFlagChoice (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `flagChoices`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.flagChoices
const call = native.flagChoicesWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["id"] = v1
break
}
case 1: {
let v1
v1 = rStr()
o1["label"] = v1
break
}
case 2: {
let v1
v1 = rStr()
o1["flag"] = v1
break
}
case 3: {
let v1
v1 = rStr()
o1["keywords"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.flagChoices = function flagChoices (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of flagChoices has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `matchPreset`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.matchPreset
const call = native.matchPresetWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.matchPreset = function matchPreset (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `missingCategories`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.missingCategories
const call = native.missingCategoriesWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.missingCategories = function missingCategories (a0, a1) {
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `geodataPreset`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.geodataPreset
const call = native.geodataPresetWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.geodataPreset = function geodataPreset (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of geodataPreset has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `conditionFamily`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.conditionFamily
const call = native.conditionFamilyWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.conditionFamily = function conditionFamily (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `newGroupCondition`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.newGroupCondition
const call = native.newGroupConditionWire
const encode = () => {
at = 0
return true
}
const decode = () => {
let t
{
const o0 = {}
const n0 = rU8()
for (let j0 = 0; j0 < n0; j0++) {
switch (rU8()) {
case 0: {
let v0
v0 = rF64()
o0["id"] = v0
break
}
case 1: {
let v0
v0 = rDyn()
o0["kind"] = v0
break
}
case 2: {
let v0
v0 = rStr()
o0["value"] = v0
break
}
case 3: {
let v0
v0 = rU8() === 1
o0["negate"] = v0
break
}
case 4: {
let v0
{
const g1 = rU8()
if (g1 === 0) v0 = undefined
else if (g1 === 2) v0 = null
else {
if (rU8() === 0) v0 = null
else {
const n2 = rU32()
const a2 = new Array(n2)
for (let i2 = 0; i2 < n2; i2++) {
let v2
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rF64()
o3["id"] = v3
break
}
case 1: {
let v3
v3 = rDyn()
o3["kind"] = v3
break
}
case 2: {
let v3
v3 = rStr()
o3["value"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v2 = o3
}
a2[i2] = v2
}
v0 = a2
}
}
}
o0["alternatives"] = v0
break
}
default: {
const k0 = rStr()
o0[k0] = rDyn()
}
}
}
t = o0
}
return t
}
module.exports.newGroupCondition = function newGroupCondition () {
  try {
    if (arguments.length <= 0 && encode() && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `conditionValues`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.conditionValues
const call = native.conditionValuesWire
const encode = (a0) => {
at = 0
if (!Array.isArray(a0)) return false
wU8(1)
wU32(a0.length)
for (let i0 = 0; i0 < a0.length; i0++) {
const v0 = a0[i0]
if (typeof v0 !== 'string') return false
wStr(v0)
}
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.conditionValues = function conditionValues (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of conditionValues has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `parseConditionValues`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.parseConditionValues
const call = native.parseConditionValuesWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
if (rU8() === 0) t = null
else {
const n1 = rU32()
const a1 = new Array(n1)
for (let i1 = 0; i1 < n1; i1++) {
let v1
v1 = rStr()
a1[i1] = v1
}
t = a1
}
}
}
return t
}
module.exports.parseConditionValues = function parseConditionValues (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of parseConditionValues has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `parseGroupConditions`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.parseGroupConditions
const call = native.parseGroupConditionsWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
if (rU8() === 0) t = null
else {
const n1 = rU32()
const a1 = new Array(n1)
for (let i1 = 0; i1 < n1; i1++) {
let v1
{
const o2 = {}
const n2 = rU8()
for (let j2 = 0; j2 < n2; j2++) {
switch (rU8()) {
case 0: {
let v2
v2 = rU8() === 1
o2["negate"] = v2
break
}
case 1: {
let v2
{
const g3 = rU8()
if (g3 === 0) v2 = undefined
else if (g3 === 2) v2 = null
else {
if (rU8() === 0) v2 = null
else {
const n4 = rU32()
const a4 = new Array(n4)
for (let i4 = 0; i4 < n4; i4++) {
let v4
{
const o5 = {}
const n5 = rU8()
for (let j5 = 0; j5 < n5; j5++) {
switch (rU8()) {
case 0: {
let v5
v5 = rF64()
o5["id"] = v5
break
}
case 1: {
let v5
v5 = rDyn()
o5["kind"] = v5
break
}
case 2: {
let v5
v5 = rStr()
o5["value"] = v5
break
}
default: {
const k5 = rStr()
o5[k5] = rDyn()
}
}
}
v4 = o5
}
a4[i4] = v4
}
v2 = a4
}
}
}
o2["alternatives"] = v2
break
}
default: {
const k2 = rStr()
o2[k2] = rDyn()
}
}
}
v1 = o2
}
a1[i1] = v1
}
t = a1
}
}
}
return t
}
module.exports.parseGroupConditions = function parseGroupConditions (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of parseGroupConditions has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `serializeGroupConditions`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.serializeGroupConditions
const call = native.serializeGroupConditionsWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.serializeGroupConditions = function serializeGroupConditions (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `groupFilterDraft`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.groupFilterDraft
const call = native.groupFilterDraftWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (a1 === undefined) {
wU8(0)
} else if (a1 === null) {
return false
wU8(2)
} else {
wU8(1)
if (typeof a1 !== 'boolean') return false
wU8(a1 ? 1 : 0)
}
return true
}
const decode = () => {
let t
{
const o0 = {}
const n0 = rU8()
for (let j0 = 0; j0 < n0; j0++) {
switch (rU8()) {
case 0: {
let v0
v0 = rF64()
o0["id"] = v0
break
}
case 1: {
let v0
v0 = rStr()
o0["source"] = v0
break
}
case 2: {
let v0
v0 = rStr()
o0["initial"] = v0
break
}
case 3: {
let v0
{
const g1 = rU8()
if (g1 === 0) v0 = undefined
else if (g1 === 2) v0 = null
else {
if (rU8() === 0) v0 = null
else {
const n2 = rU32()
const a2 = new Array(n2)
for (let i2 = 0; i2 < n2; i2++) {
let v2
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rU8() === 1
o3["negate"] = v3
break
}
case 1: {
let v3
{
const g4 = rU8()
if (g4 === 0) v3 = undefined
else if (g4 === 2) v3 = null
else {
if (rU8() === 0) v3 = null
else {
const n5 = rU32()
const a5 = new Array(n5)
for (let i5 = 0; i5 < n5; i5++) {
let v5
{
const o6 = {}
const n6 = rU8()
for (let j6 = 0; j6 < n6; j6++) {
switch (rU8()) {
case 0: {
let v6
v6 = rF64()
o6["id"] = v6
break
}
case 1: {
let v6
v6 = rDyn()
o6["kind"] = v6
break
}
case 2: {
let v6
v6 = rStr()
o6["value"] = v6
break
}
default: {
const k6 = rStr()
o6[k6] = rDyn()
}
}
}
v5 = o6
}
a5[i5] = v5
}
v3 = a5
}
}
}
o3["alternatives"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v2 = o3
}
a2[i2] = v2
}
v0 = a2
}
}
}
o0["rows"] = v0
break
}
case 4: {
let v0
v0 = rU8() === 1
o0["advanced"] = v0
break
}
default: {
const k0 = rStr()
o0[k0] = rDyn()
}
}
}
t = o0
}
return t
}
module.exports.groupFilterDraft = function groupFilterDraft (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of groupFilterDraft has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `groupFilterText`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.groupFilterText
const call = native.groupFilterTextWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.groupFilterText = function groupFilterText (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `groupFilterTexts`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.groupFilterTexts
const call = native.groupFilterTextsWire
const encode = (a0) => {
at = 0
if (!Array.isArray(a0)) return false
wU8(1)
wU32(a0.length)
for (let i0 = 0; i0 < a0.length; i0++) {
const v0 = a0[i0]
if (v0 === null || typeof v0 !== 'object' || Array.isArray(v0)) return false
{
const at1 = at
wU8(0)
let n1 = 0, r1 = 0
for (const k1 in v0) {
switch (k1) {
case "id": {
wU8(0)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'number') return false
wF64(f1)
break
}
case "source": {
wU8(1)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'string') return false
wStr(f1)
break
}
case "initial": {
wU8(2)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'string') return false
wStr(f1)
break
}
case "rows": {
wU8(3)
r1++
const f1 = v0[k1]
if (f1 === undefined) {
return false
wU8(0)
} else if (f1 === null) {
wU8(2)
} else {
wU8(1)
if (!Array.isArray(f1)) return false
wU8(1)
wU32(f1.length)
for (let i2 = 0; i2 < f1.length; i2++) {
const v2 = f1[i2]
if (v2 === null || typeof v2 !== 'object' || Array.isArray(v2)) return false
{
const at3 = at
wU8(0)
let n3 = 0, r3 = 0
for (const k3 in v2) {
switch (k3) {
case "negate": {
wU8(0)
r3++
const f3 = v2[k3]
if (typeof f3 !== 'boolean') return false
wU8(f3 ? 1 : 0)
break
}
case "alternatives": {
wU8(1)
const f3 = v2[k3]
if (f3 === undefined) {
wU8(0)
} else if (f3 === null) {
return false
wU8(2)
} else {
wU8(1)
if (!Array.isArray(f3)) return false
wU8(1)
wU32(f3.length)
for (let i4 = 0; i4 < f3.length; i4++) {
const v4 = f3[i4]
if (v4 === null || typeof v4 !== 'object' || Array.isArray(v4)) return false
{
const at5 = at
wU8(0)
let n5 = 0, r5 = 0
for (const k5 in v4) {
switch (k5) {
case "id": {
wU8(0)
const f5 = v4[k5]
if (!wDyn(f5)) return false
break
}
case "kind": {
wU8(1)
const f5 = v4[k5]
if (!wDyn(f5)) return false
break
}
case "value": {
wU8(2)
const f5 = v4[k5]
if (!wDyn(f5)) return false
break
}
default: return false
}
if (++n5 > 255) return false
}
if (r5 !== 0) return false
iv.setUint8(at5, n5)
}
}
}
break
}
default: return false
}
if (++n3 > 255) return false
}
if (r3 !== 1) return false
iv.setUint8(at3, n3)
}
}
}
break
}
case "advanced": {
wU8(4)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'boolean') return false
wU8(f1 ? 1 : 0)
break
}
default: return false
}
if (++n1 > 255) return false
}
if (r1 !== 5) return false
iv.setUint8(at1, n1)
}
}
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.groupFilterTexts = function groupFilterTexts (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `reconcileGroupFilters`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.reconcileGroupFilters
const call = native.reconcileGroupFiltersWire
const encode = (a0, a1) => {
at = 0
if (!Array.isArray(a0)) return false
wU8(1)
wU32(a0.length)
for (let i0 = 0; i0 < a0.length; i0++) {
const v0 = a0[i0]
if (v0 === null || typeof v0 !== 'object' || Array.isArray(v0)) return false
{
const at1 = at
wU8(0)
let n1 = 0, r1 = 0
for (const k1 in v0) {
switch (k1) {
case "id": {
wU8(0)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'number') return false
wF64(f1)
break
}
case "source": {
wU8(1)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'string') return false
wStr(f1)
break
}
case "initial": {
wU8(2)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'string') return false
wStr(f1)
break
}
case "rows": {
wU8(3)
r1++
const f1 = v0[k1]
if (f1 === undefined) {
return false
wU8(0)
} else if (f1 === null) {
wU8(2)
} else {
wU8(1)
if (!Array.isArray(f1)) return false
wU8(1)
wU32(f1.length)
for (let i2 = 0; i2 < f1.length; i2++) {
const v2 = f1[i2]
if (v2 === null || typeof v2 !== 'object' || Array.isArray(v2)) return false
{
const at3 = at
wU8(0)
let n3 = 0, r3 = 0
for (const k3 in v2) {
switch (k3) {
case "negate": {
wU8(0)
r3++
const f3 = v2[k3]
if (typeof f3 !== 'boolean') return false
wU8(f3 ? 1 : 0)
break
}
case "alternatives": {
wU8(1)
const f3 = v2[k3]
if (f3 === undefined) {
wU8(0)
} else if (f3 === null) {
return false
wU8(2)
} else {
wU8(1)
if (!Array.isArray(f3)) return false
wU8(1)
wU32(f3.length)
for (let i4 = 0; i4 < f3.length; i4++) {
const v4 = f3[i4]
if (v4 === null || typeof v4 !== 'object' || Array.isArray(v4)) return false
{
const at5 = at
wU8(0)
let n5 = 0, r5 = 0
for (const k5 in v4) {
switch (k5) {
case "id": {
wU8(0)
const f5 = v4[k5]
if (!wDyn(f5)) return false
break
}
case "kind": {
wU8(1)
const f5 = v4[k5]
if (!wDyn(f5)) return false
break
}
case "value": {
wU8(2)
const f5 = v4[k5]
if (!wDyn(f5)) return false
break
}
default: return false
}
if (++n5 > 255) return false
}
if (r5 !== 0) return false
iv.setUint8(at5, n5)
}
}
}
break
}
default: return false
}
if (++n3 > 255) return false
}
if (r3 !== 1) return false
iv.setUint8(at3, n3)
}
}
}
break
}
case "advanced": {
wU8(4)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'boolean') return false
wU8(f1 ? 1 : 0)
break
}
default: return false
}
if (++n1 > 255) return false
}
if (r1 !== 5) return false
iv.setUint8(at1, n1)
}
}
if (!Array.isArray(a1)) return false
wU8(1)
wU32(a1.length)
for (let i0 = 0; i0 < a1.length; i0++) {
const v0 = a1[i0]
if (typeof v0 !== 'string') return false
wStr(v0)
}
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rF64()
o1["id"] = v1
break
}
case 1: {
let v1
v1 = rStr()
o1["source"] = v1
break
}
case 2: {
let v1
v1 = rStr()
o1["initial"] = v1
break
}
case 3: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
if (rU8() === 0) v1 = null
else {
const n3 = rU32()
const a3 = new Array(n3)
for (let i3 = 0; i3 < n3; i3++) {
let v3
{
const o4 = {}
const n4 = rU8()
for (let j4 = 0; j4 < n4; j4++) {
switch (rU8()) {
case 0: {
let v4
v4 = rU8() === 1
o4["negate"] = v4
break
}
case 1: {
let v4
{
const g5 = rU8()
if (g5 === 0) v4 = undefined
else if (g5 === 2) v4 = null
else {
if (rU8() === 0) v4 = null
else {
const n6 = rU32()
const a6 = new Array(n6)
for (let i6 = 0; i6 < n6; i6++) {
let v6
{
const o7 = {}
const n7 = rU8()
for (let j7 = 0; j7 < n7; j7++) {
switch (rU8()) {
case 0: {
let v7
v7 = rDyn()
o7["id"] = v7
break
}
case 1: {
let v7
v7 = rDyn()
o7["kind"] = v7
break
}
case 2: {
let v7
v7 = rDyn()
o7["value"] = v7
break
}
default: {
const k7 = rStr()
o7[k7] = rDyn()
}
}
}
v6 = o7
}
a6[i6] = v6
}
v4 = a6
}
}
}
o4["alternatives"] = v4
break
}
default: {
const k4 = rStr()
o4[k4] = rDyn()
}
}
}
v3 = o4
}
a3[i3] = v3
}
v1 = a3
}
}
}
o1["rows"] = v1
break
}
case 4: {
let v1
v1 = rU8() === 1
o1["advanced"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.reconcileGroupFilters = function reconcileGroupFilters (a0, a1) {
  if (illFormed(a1)) throw new TypeError('argument 2 of reconcileGroupFilters has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `newGroupFilter`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.newGroupFilter
const call = native.newGroupFilterWire
const encode = () => {
at = 0
return true
}
const decode = () => {
let t
{
const o0 = {}
const n0 = rU8()
for (let j0 = 0; j0 < n0; j0++) {
switch (rU8()) {
case 0: {
let v0
v0 = rF64()
o0["id"] = v0
break
}
case 1: {
let v0
v0 = rStr()
o0["source"] = v0
break
}
case 2: {
let v0
v0 = rStr()
o0["initial"] = v0
break
}
case 3: {
let v0
{
const g1 = rU8()
if (g1 === 0) v0 = undefined
else if (g1 === 2) v0 = null
else {
if (rU8() === 0) v0 = null
else {
const n2 = rU32()
const a2 = new Array(n2)
for (let i2 = 0; i2 < n2; i2++) {
let v2
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rU8() === 1
o3["negate"] = v3
break
}
case 1: {
let v3
{
const g4 = rU8()
if (g4 === 0) v3 = undefined
else if (g4 === 2) v3 = null
else {
if (rU8() === 0) v3 = null
else {
const n5 = rU32()
const a5 = new Array(n5)
for (let i5 = 0; i5 < n5; i5++) {
let v5
{
const o6 = {}
const n6 = rU8()
for (let j6 = 0; j6 < n6; j6++) {
switch (rU8()) {
case 0: {
let v6
v6 = rF64()
o6["id"] = v6
break
}
case 1: {
let v6
v6 = rDyn()
o6["kind"] = v6
break
}
case 2: {
let v6
v6 = rStr()
o6["value"] = v6
break
}
default: {
const k6 = rStr()
o6[k6] = rDyn()
}
}
}
v5 = o6
}
a5[i5] = v5
}
v3 = a5
}
}
}
o3["alternatives"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v2 = o3
}
a2[i2] = v2
}
v0 = a2
}
}
}
o0["rows"] = v0
break
}
case 4: {
let v0
v0 = rU8() === 1
o0["advanced"] = v0
break
}
default: {
const k0 = rStr()
o0[k0] = rDyn()
}
}
}
t = o0
}
return t
}
module.exports.newGroupFilter = function newGroupFilter () {
  try {
    if (arguments.length <= 0 && encode() && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `groupAdmits`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.groupAdmits
const call = native.groupAdmitsWire
const encode = (a0, a1) => {
at = 0
if (!Array.isArray(a0)) return false
wU8(1)
wU32(a0.length)
for (let i0 = 0; i0 < a0.length; i0++) {
const v0 = a0[i0]
if (typeof v0 !== 'string') return false
wStr(v0)
}
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.groupAdmits = function groupAdmits (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of groupAdmits has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `readGroupEntries`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.readGroupEntries
const call = native.readGroupEntriesWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["name"] = v1
break
}
case 1: {
let v1
v1 = rStr()
o1["written"] = v1
break
}
case 2: {
let v1
if (rU8() === 0) v1 = null
else {
const n2 = rU32()
const a2 = new Array(n2)
for (let i2 = 0; i2 < n2; i2++) {
let v2
v2 = rStr()
a2[i2] = v2
}
v1 = a2
}
o1["filters"] = v1
break
}
case 3: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["policy"] = v1
break
}
case 4: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["default"] = v1
break
}
case 5: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["final"] = v1
break
}
case 6: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["exit"] = v1
break
}
case 7: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["interrupt"] = v1
break
}
case 8: {
let v1
v1 = rF64()
o1["from"] = v1
break
}
case 9: {
let v1
v1 = rF64()
o1["to"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.readGroupEntries = function readGroupEntries (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of readGroupEntries has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `quoteName`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.quoteName
const call = native.quoteNameWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.quoteName = function quoteName (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of quoteName has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `isWritableName`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.isWritableName
const call = native.isWritableNameWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.isWritableName = function isWritableName (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of isWritableName has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `groupNameProblem`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.groupNameProblem
const call = native.groupNameProblemWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.groupNameProblem = function groupNameProblem (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of groupNameProblem has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `writeGroupEntry`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.writeGroupEntry
const call = native.writeGroupEntryWire
const encode = (a0, a1, a2) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'string') return false
wStr(a1)
if (!wDyn(a2)) return false
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.writeGroupEntry = function writeGroupEntry (a0, a1, a2) {
  if (illFormed(a0)) throw new TypeError('argument 1 of writeGroupEntry has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of writeGroupEntry has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `nameText`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.nameText
const call = native.nameTextWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.nameText = function nameText (a0, a1) {
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `nestedIn`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.nestedIn
const call = native.nestedInWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
v0 = rStr()
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.nestedIn = function nestedIn (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `exactTokens`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.exactTokens
const call = native.exactTokensWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.exactTokens = function exactTokens (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of exactTokens has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `namedIn`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.namedIn
const call = native.namedInWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
v0 = rStr()
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.namedIn = function namedIn (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `classifyFilters`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.classifyFilters
const call = native.classifyFiltersWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.classifyFilters = function classifyFilters (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `describeFilters`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.describeFilters
const call = native.describeFiltersWire
const encode = (a0) => {
at = 0
if (!Array.isArray(a0)) return false
wU8(1)
wU32(a0.length)
for (let i0 = 0; i0 < a0.length; i0++) {
const v0 = a0[i0]
if (typeof v0 !== 'string') return false
wStr(v0)
}
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.describeFilters = function describeFilters (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of describeFilters has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `citingGroups`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.citingGroups
const call = native.citingGroupsWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.citingGroups = function citingGroups (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of citingGroups has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of citingGroups has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `namedInExpression`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.namedInExpression
const call = native.namedInExpressionWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.namedInExpression = function namedInExpression (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of namedInExpression has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of namedInExpression has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `groupsNamingTag`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.groupsNamingTag
const call = native.groupsNamingTagWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.groupsNamingTag = function groupsNamingTag (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of groupsNamingTag has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of groupsNamingTag has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `addSubtagsToGroup`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.addSubtagsToGroup
const call = native.addSubtagsToGroupWire
const encode = (a0, a1, a2) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'string') return false
wStr(a1)
if (!Array.isArray(a2)) return false
wU8(1)
wU32(a2.length)
for (let i0 = 0; i0 < a2.length; i0++) {
const v0 = a2[i0]
if (typeof v0 !== 'string') return false
wStr(v0)
}
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.addSubtagsToGroup = function addSubtagsToGroup (a0, a1, a2) {
  if (illFormed(a0)) throw new TypeError('argument 1 of addSubtagsToGroup has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of addSubtagsToGroup has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a2)) throw new TypeError('argument 3 of addSubtagsToGroup has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `addNodesToGroup`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.addNodesToGroup
const call = native.addNodesToGroupWire
const encode = (a0, a1, a2) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'string') return false
wStr(a1)
if (!Array.isArray(a2)) return false
wU8(1)
wU32(a2.length)
for (let i0 = 0; i0 < a2.length; i0++) {
const v0 = a2[i0]
if (typeof v0 !== 'string') return false
wStr(v0)
}
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.addNodesToGroup = function addNodesToGroup (a0, a1, a2) {
  if (illFormed(a0)) throw new TypeError('argument 1 of addNodesToGroup has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of addNodesToGroup has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a2)) throw new TypeError('argument 3 of addNodesToGroup has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `removeSubtagsFromGroup`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.removeSubtagsFromGroup
const call = native.removeSubtagsFromGroupWire
const encode = (a0, a1, a2) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'string') return false
wStr(a1)
if (!Array.isArray(a2)) return false
wU8(1)
wU32(a2.length)
for (let i0 = 0; i0 < a2.length; i0++) {
const v0 = a2[i0]
if (typeof v0 !== 'string') return false
wStr(v0)
}
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.removeSubtagsFromGroup = function removeSubtagsFromGroup (a0, a1, a2) {
  if (illFormed(a0)) throw new TypeError('argument 1 of removeSubtagsFromGroup has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of removeSubtagsFromGroup has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a2)) throw new TypeError('argument 3 of removeSubtagsFromGroup has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `ruleCondition`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.ruleCondition
const call = native.ruleConditionWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.ruleCondition = function ruleCondition (a0, a1) {
  if (illFormed(a1)) throw new TypeError('argument 2 of ruleCondition has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `isGlob`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.isGlob
const call = native.isGlobWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.isGlob = function isGlob (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of isGlob has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `includedBy`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.includedBy
const call = native.includedByWire
const encode = (a0, a1) => {
at = 0
if (!Array.isArray(a0)) return false
wU8(1)
wU32(a0.length)
for (let i0 = 0; i0 < a0.length; i0++) {
const v0 = a0[i0]
if (typeof v0 !== 'string') return false
wStr(v0)
}
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.includedBy = function includedBy (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of includedBy has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of includedBy has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `resolveIncludePath`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.resolveIncludePath
const call = native.resolveIncludePathWire
const encode = (a0, a1) => {
at = 0
if (a0 === undefined) {
wU8(0)
} else if (a0 === null) {
return false
wU8(2)
} else {
wU8(1)
if (typeof a0 !== 'string') return false
wStr(a0)
}
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.resolveIncludePath = function resolveIncludePath (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of resolveIncludePath has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of resolveIncludePath has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `newSourcePathProblem`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.newSourcePathProblem
const call = native.newSourcePathProblemWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.newSourcePathProblem = function newSourcePathProblem (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of newSourcePathProblem has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `namePatterns`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.namePatterns
const call = native.namePatternsWire
const encode = (a0) => {
at = 0
if (!Array.isArray(a0)) return false
wU8(1)
wU32(a0.length)
for (let i0 = 0; i0 < a0.length; i0++) {
const v0 = a0[i0]
if (typeof v0 !== 'string') return false
wStr(v0)
}
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["pattern"] = v1
break
}
case 1: {
let v1
v1 = rStr()
o1["prefix"] = v1
break
}
case 2: {
let v1
v1 = rStr()
o1["suffix"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.namePatterns = function namePatterns (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of namePatterns has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `newSourceNameProblem`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.newSourceNameProblem
const call = native.newSourceNameProblemWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.newSourceNameProblem = function newSourceNameProblem (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of newSourceNameProblem has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `sourceAt`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.sourceAt
const call = native.sourceAtWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.sourceAt = function sourceAt (a0, a1) {
  if (illFormed(a1)) throw new TypeError('argument 2 of sourceAt has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `protocolName`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.protocolName
const call = native.protocolNameWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.protocolName = function protocolName (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `emptyNode`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.emptyNode
const call = native.emptyNodeWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const o0 = {}
for (let i0 = 0; i0 < n0; i0++) {
const k0 = rStr()
let v0
v0 = rStr()
o0[k0] = v0
}
t = o0
}
return t
}
module.exports.emptyNode = function emptyNode (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `parseNodeLink`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.parseNodeLink
const call = native.parseNodeLinkWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
{
const o0 = {}
const n0 = rU8()
for (let j0 = 0; j0 < n0; j0++) {
switch (rU8()) {
case 0: {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["original"] = v1
break
}
case 1: {
let v1
if (rU8() === 0) v1 = null
else {
const n2 = rU32()
const o2 = {}
for (let i2 = 0; i2 < n2; i2++) {
const k2 = rStr()
let v2
v2 = rStr()
o2[k2] = v2
}
v1 = o2
}
o1["initial"] = v1
break
}
case 2: {
let v1
v1 = rStr()
o1["query"] = v1
break
}
case 3: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
if (rU8() === 0) v1 = null
else {
const n3 = rU32()
const o3 = {}
for (let i3 = 0; i3 < n3; i3++) {
const k3 = rStr()
let v3
v3 = rDyn()
o3[k3] = v3
}
v1 = o3
}
}
}
o1["json"] = v1
break
}
case 4: {
let v1
v1 = rStr()
o1["tail"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
o0["document"] = v0
break
}
case 1: {
let v0
if (rU8() === 0) v0 = null
else {
const n1 = rU32()
const o1 = {}
for (let i1 = 0; i1 < n1; i1++) {
const k1 = rStr()
let v1
v1 = rStr()
o1[k1] = v1
}
v0 = o1
}
o0["fields"] = v0
break
}
default: {
const k0 = rStr()
o0[k0] = rDyn()
}
}
}
t = o0
}
return t
}
module.exports.parseNodeLink = function parseNodeLink (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of parseNodeLink has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `nodeLink`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.nodeLink
const call = native.nodeLinkWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.nodeLink = function nodeLink (a0, a1) {
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `securityOptions`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.securityOptions
const call = native.securityOptionsWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.securityOptions = function securityOptions (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `usesTls`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.usesTls
const call = native.usesTlsWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.usesTls = function usesTls (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `manualFields`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.manualFields
const call = native.manualFieldsWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["key"] = v1
break
}
case 1: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
if (rU8() === 0) v1 = null
else {
const n3 = rU32()
const a3 = new Array(n3)
for (let i3 = 0; i3 < n3; i3++) {
let v3
v3 = rStr()
a3[i3] = v3
}
v1 = a3
}
}
}
o1["options"] = v1
break
}
case 2: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["flag"] = v1
break
}
case 3: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rU8() === 1
}
}
o1["required"] = v1
break
}
case 4: {
let v1
v1 = rDyn()
o1["range"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.manualFields = function manualFields (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `advancedFields`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.advancedFields
const call = native.advancedFieldsWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["key"] = v1
break
}
case 1: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
if (rU8() === 0) v1 = null
else {
const n3 = rU32()
const a3 = new Array(n3)
for (let i3 = 0; i3 < n3; i3++) {
let v3
v3 = rStr()
a3[i3] = v3
}
v1 = a3
}
}
}
o1["options"] = v1
break
}
case 2: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["flag"] = v1
break
}
case 3: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rU8() === 1
}
}
o1["required"] = v1
break
}
case 4: {
let v1
v1 = rDyn()
o1["range"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.advancedFields = function advancedFields (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `settle`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.settle
const call = native.settleWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const o0 = {}
for (let i0 = 0; i0 < n0; i0++) {
const k0 = rStr()
let v0
v0 = rStr()
o0[k0] = v0
}
t = o0
}
return t
}
module.exports.settle = function settle (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `manualInvalid`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.manualInvalid
const call = native.manualInvalidWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
v0 = rStr()
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.manualInvalid = function manualInvalid (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `manualLink`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.manualLink
const call = native.manualLinkWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.manualLink = function manualLink (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `isNodeLink`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.isNodeLink
const call = native.isNodeLinkWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.isNodeLink = function isNodeLink (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of isNodeLink has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `readNodeEntries`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.readNodeEntries
const call = native.readNodeEntriesWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["name"] = v1
break
}
case 1: {
let v1
v1 = rStr()
o1["link"] = v1
break
}
case 2: {
let v1
v1 = rF64()
o1["from"] = v1
break
}
case 3: {
let v1
v1 = rF64()
o1["to"] = v1
break
}
case 4: {
let v1
v1 = rF64()
o1["line"] = v1
break
}
case 5: {
let v1
v1 = rF64()
o1["nameTo"] = v1
break
}
case 6: {
let v1
v1 = rF64()
o1["linkFrom"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.readNodeEntries = function readNodeEntries (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of readNodeEntries has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `readChainEntries`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.readChainEntries
const call = native.readChainEntriesWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["name"] = v1
break
}
case 1: {
let v1
if (rU8() === 0) v1 = null
else {
const n2 = rU32()
const a2 = new Array(n2)
for (let i2 = 0; i2 < n2; i2++) {
let v2
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rStr()
o3["name"] = v3
break
}
case 1: {
let v3
v3 = rF64()
o3["from"] = v3
break
}
case 2: {
let v3
v3 = rF64()
o3["to"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v2 = o3
}
a2[i2] = v2
}
v1 = a2
}
o1["hops"] = v1
break
}
case 2: {
let v1
v1 = rF64()
o1["from"] = v1
break
}
case 3: {
let v1
v1 = rF64()
o1["to"] = v1
break
}
case 4: {
let v1
v1 = rF64()
o1["line"] = v1
break
}
case 5: {
let v1
v1 = rF64()
o1["nameTo"] = v1
break
}
case 6: {
let v1
v1 = rF64()
o1["hopsFrom"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.readChainEntries = function readChainEntries (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of readChainEntries has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `nodeReferenced`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.nodeReferenced
const call = native.nodeReferencedWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.nodeReferenced = function nodeReferenced (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of nodeReferenced has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of nodeReferenced has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `nodeReferrers`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.nodeReferrers
const call = native.nodeReferrersWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["kind"] = v1
break
}
case 1: {
let v1
v1 = rStr()
o1["name"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.nodeReferrers = function nodeReferrers (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of nodeReferrers has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of nodeReferrers has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `removeChainEntry`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.removeChainEntry
const call = native.removeChainEntryWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.removeChainEntry = function removeChainEntry (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of removeChainEntry has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `chainHopCandidate`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.chainHopCandidate
const call = native.chainHopCandidateWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.chainHopCandidate = function chainHopCandidate (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `tunnelDialable`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.tunnelDialable
const call = native.tunnelDialableWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.tunnelDialable = function tunnelDialable (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `chainHopProblem`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.chainHopProblem
const call = native.chainHopProblemWire
const encode = (a0, a1, a2) => {
at = 0
if (!wDyn(a0)) return false
if (typeof a1 !== 'string') return false
wStr(a1)
if (typeof a2 !== 'number') return false
wF64(a2)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.chainHopProblem = function chainHopProblem (a0, a1, a2) {
  if (illFormed(a1)) throw new TypeError('argument 2 of chainHopProblem has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `chainUdpProblem`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.chainUdpProblem
const call = native.chainUdpProblemWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.chainUdpProblem = function chainUdpProblem (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `writeChainEntry`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.writeChainEntry
const call = native.writeChainEntryWire
const encode = (a0, a1, a2) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
if (!wDyn(a2)) return false
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.writeChainEntry = function writeChainEntry (a0, a1, a2) {
  if (illFormed(a0)) throw new TypeError('argument 1 of writeChainEntry has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `appendChainEntry`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.appendChainEntry
const call = native.appendChainEntryWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.appendChainEntry = function appendChainEntry (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of appendChainEntry has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `writeNodeEntry`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.writeNodeEntry
const call = native.writeNodeEntryWire
const encode = (a0, a1, a2) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
if (!wDyn(a2)) return false
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.writeNodeEntry = function writeNodeEntry (a0, a1, a2) {
  if (illFormed(a0)) throw new TypeError('argument 1 of writeNodeEntry has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `readMode`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.readMode
const call = native.readModeWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.readMode = function readMode (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of readMode has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `writeMode`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.writeMode
const call = native.writeModeWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.writeMode = function writeMode (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of writeMode has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `sameMode`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.sameMode
const call = native.sameModeWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.sameMode = function sameMode (a0, a1) {
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `modeRuleLines`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.modeRuleLines
const call = native.modeRuleLinesWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
v0 = rF64()
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.modeRuleLines = function modeRuleLines (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of modeRuleLines has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `nodeProbeSupport`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.nodeProbeSupport
const call = native.nodeProbeSupportWire
const encode = (a0) => {
at = 0
if (a0 === undefined) {
return false
wU8(0)
} else if (a0 === null) {
wU8(2)
} else {
wU8(1)
if (typeof a0 !== 'string') return false
wStr(a0)
}
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.nodeProbeSupport = function nodeProbeSupport (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of nodeProbeSupport has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `supportsNodeProbe`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.supportsNodeProbe
const call = native.supportsNodeProbeWire
const encode = (a0, a1, a2) => {
at = 0
if (a0 === undefined) {
return false
wU8(0)
} else if (a0 === null) {
wU8(2)
} else {
wU8(1)
if (typeof a0 !== 'string') return false
wStr(a0)
}
if (!wDyn(a1)) return false
if (!wDyn(a2)) return false
return true
}
const decode = () => {
let t
t = rU8() === 1
return t
}
module.exports.supportsNodeProbe = function supportsNodeProbe (a0, a1, a2) {
  if (illFormed(a0)) throw new TypeError('argument 1 of supportsNodeProbe has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `regionFlag`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.regionFlag
const call = native.regionFlagWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.regionFlag = function regionFlag (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of regionFlag has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `matchRegion`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.matchRegion
const call = native.matchRegionWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.matchRegion = function matchRegion (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of matchRegion has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `sourceFor`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.sourceFor
const call = native.sourceForWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.sourceFor = function sourceFor (a0, a1) {
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `ruleAnchor`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.ruleAnchor
const call = native.ruleAnchorWire
const encode = (a0, a1, a2) => {
at = 0
if (!wDyn(a0)) return false
if (!wDyn(a1)) return false
if (!wDyn(a2)) return false
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rF64()
o1["from"] = v1
break
}
case 1: {
let v1
v1 = rF64()
o1["to"] = v1
break
}
case 2: {
let v1
v1 = rStr()
o1["indent"] = v1
break
}
case 3: {
let v1
v1 = rStr()
o1["text"] = v1
break
}
case 4: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rF64()
o3["from"] = v3
break
}
case 1: {
let v3
v3 = rF64()
o3["to"] = v3
break
}
case 2: {
let v3
{
const g4 = rU8()
if (g4 === 0) v3 = undefined
else if (g4 === 2) v3 = null
else {
v3 = rU8() === 1
}
}
o3["foldCase"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v1 = o3
}
}
}
o1["target"] = v1
break
}
case 5: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rF64()
o3["from"] = v3
break
}
case 1: {
let v3
v3 = rF64()
o3["to"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v1 = o3
}
}
}
o1["condition"] = v1
break
}
case 6: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["open"] = v1
break
}
case 7: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["close"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
t = o1
}
}
}
return t
}
module.exports.ruleAnchor = function ruleAnchor (a0, a1, a2) {
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `dnsRuleTarget`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.dnsRuleTarget
const call = native.dnsRuleTargetWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.dnsRuleTarget = function dnsRuleTarget (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `dnsRuleAnchor`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.dnsRuleAnchor
const call = native.dnsRuleAnchorWire
const encode = (a0, a1, a2, a3) => {
at = 0
if (!wDyn(a0)) return false
if (!wDyn(a1)) return false
if (typeof a2 !== 'string') return false
wStr(a2)
if (!wDyn(a3)) return false
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rF64()
o1["from"] = v1
break
}
case 1: {
let v1
v1 = rF64()
o1["to"] = v1
break
}
case 2: {
let v1
v1 = rStr()
o1["indent"] = v1
break
}
case 3: {
let v1
v1 = rStr()
o1["text"] = v1
break
}
case 4: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rF64()
o3["from"] = v3
break
}
case 1: {
let v3
v3 = rF64()
o3["to"] = v3
break
}
case 2: {
let v3
{
const g4 = rU8()
if (g4 === 0) v3 = undefined
else if (g4 === 2) v3 = null
else {
v3 = rU8() === 1
}
}
o3["foldCase"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v1 = o3
}
}
}
o1["target"] = v1
break
}
case 5: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rF64()
o3["from"] = v3
break
}
case 1: {
let v3
v3 = rF64()
o3["to"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v1 = o3
}
}
}
o1["condition"] = v1
break
}
case 6: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["open"] = v1
break
}
case 7: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["close"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
t = o1
}
}
}
return t
}
module.exports.dnsRuleAnchor = function dnsRuleAnchor (a0, a1, a2, a3) {
  if (illFormed(a2)) throw new TypeError('argument 3 of dnsRuleAnchor has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 4 && encode(a0, a1, a2, a3) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `dnsListEnd`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.dnsListEnd
const call = native.dnsListEndWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.dnsListEnd = function dnsListEnd (a0, a1) {
  if (illFormed(a1)) throw new TypeError('argument 2 of dnsListEnd has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `dnsUpstreams`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.dnsUpstreams
const call = native.dnsUpstreamsWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["name"] = v1
break
}
case 1: {
let v1
v1 = rStr()
o1["address"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.dnsUpstreams = function dnsUpstreams (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of dnsUpstreams has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `dnsUpstreamNames`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.dnsUpstreamNames
const call = native.dnsUpstreamNamesWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
v0 = rStr()
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.dnsUpstreamNames = function dnsUpstreamNames (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of dnsUpstreamNames has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `ruleLine`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.ruleLine
const call = native.ruleLineWire
const encode = (a0, a1, a2) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'string') return false
wStr(a1)
if (a2 === undefined) {
wU8(0)
} else if (a2 === null) {
return false
wU8(2)
} else {
wU8(1)
if (typeof a2 !== 'boolean') return false
wU8(a2 ? 1 : 0)
}
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.ruleLine = function ruleLine (a0, a1, a2) {
  if (illFormed(a0)) throw new TypeError('argument 1 of ruleLine has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of ruleLine has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `addRule`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.addRule
const call = native.addRuleWire
const encode = (a0, a1, a2, a3, a4) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
if (typeof a2 !== 'string') return false
wStr(a2)
if (typeof a3 !== 'string') return false
wStr(a3)
if (typeof a4 !== 'boolean') return false
wU8(a4 ? 1 : 0)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.addRule = function addRule (a0, a1, a2, a3, a4) {
  if (illFormed(a0)) throw new TypeError('argument 1 of addRule has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a2)) throw new TypeError('argument 3 of addRule has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a3)) throw new TypeError('argument 4 of addRule has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 5 && encode(a0, a1, a2, a3, a4) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `replaceRuleTarget`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.replaceRuleTarget
const call = native.replaceRuleTargetWire
const encode = (a0, a1, a2, a3) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
if (typeof a2 !== 'string') return false
wStr(a2)
if (typeof a3 !== 'boolean') return false
wU8(a3 ? 1 : 0)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.replaceRuleTarget = function replaceRuleTarget (a0, a1, a2, a3) {
  if (illFormed(a0)) throw new TypeError('argument 1 of replaceRuleTarget has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a2)) throw new TypeError('argument 3 of replaceRuleTarget has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 4 && encode(a0, a1, a2, a3) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `removeRule`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.removeRule
const call = native.removeRuleWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.removeRule = function removeRule (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of removeRule has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `replaceRule`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.replaceRule
const call = native.replaceRuleWire
const encode = (a0, a1, a2, a3, a4) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
if (typeof a2 !== 'string') return false
wStr(a2)
if (typeof a3 !== 'string') return false
wStr(a3)
if (typeof a4 !== 'boolean') return false
wU8(a4 ? 1 : 0)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.replaceRule = function replaceRule (a0, a1, a2, a3, a4) {
  if (illFormed(a0)) throw new TypeError('argument 1 of replaceRule has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a2)) throw new TypeError('argument 3 of replaceRule has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a3)) throw new TypeError('argument 4 of replaceRule has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 5 && encode(a0, a1, a2, a3, a4) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `addFallback`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.addFallback
const call = native.addFallbackWire
const encode = (a0, a1, a2) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
if (typeof a2 !== 'string') return false
wStr(a2)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.addFallback = function addFallback (a0, a1, a2) {
  if (illFormed(a0)) throw new TypeError('argument 1 of addFallback has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a2)) throw new TypeError('argument 3 of addFallback has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `numberSetting`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.numberSetting
const call = native.numberSettingWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rU8() === 1
return t
}
module.exports.numberSetting = function numberSetting (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `settingValue`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.settingValue
const call = native.settingValueWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.settingValue = function settingValue (a0, a1) {
  if (illFormed(a1)) throw new TypeError('argument 2 of settingValue has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `serializeSetting`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.serializeSetting
const call = native.serializeSettingWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (typeof a1 !== 'string') return false
wStr(a1)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.serializeSetting = function serializeSetting (a0, a1) {
  if (illFormed(a1)) throw new TypeError('argument 2 of serializeSetting has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `writeSettings`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.writeSettings
const call = native.writeSettingsWire
const encode = (a0, a1, a2, a3) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
if (typeof a2 !== 'number') return false
wF64(a2)
if (!wDyn(a3)) return false
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.writeSettings = function writeSettings (a0, a1, a2, a3) {
  if (illFormed(a0)) throw new TypeError('argument 1 of writeSettings has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 4 && encode(a0, a1, a2, a3) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `allGroupNames`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.allGroupNames
const call = native.allGroupNamesWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.allGroupNames = function allGroupNames (a0, a1) {
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `allUpstreamNames`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.allUpstreamNames
const call = native.allUpstreamNamesWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.allUpstreamNames = function allUpstreamNames (a0, a1) {
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `redacted`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.redacted
const call = native.redactedWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.redacted = function redacted (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `fileName`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.fileName
const call = native.fileNameWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.fileName = function fileName (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `validationSources`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.validationSources
const call = native.validationSourcesWire
const encode = (a0, a1) => {
at = 0
if (!wDyn(a0)) return false
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.validationSources = function validationSources (a0, a1) {
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `restartSettings`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.restartSettings
const call = native.restartSettingsWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["key"] = v1
break
}
case 1: {
let v1
v1 = rStr()
o1["sourceId"] = v1
break
}
case 2: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rF64()
}
}
o1["line"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.restartSettings = function restartSettings (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `restartRequired`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.restartRequired
const call = native.restartRequiredWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.restartRequired = function restartRequired (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `unwrittenRestartSettings`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.unwrittenRestartSettings
const call = native.unwrittenRestartSettingsWire
const encode = (a0, a1) => {
at = 0
if (!Array.isArray(a0)) return false
wU8(1)
wU32(a0.length)
for (let i0 = 0; i0 < a0.length; i0++) {
const v0 = a0[i0]
if (v0 === null || typeof v0 !== 'object' || Array.isArray(v0)) return false
{
const at1 = at
wU8(0)
let n1 = 0, r1 = 0
for (const k1 in v0) {
switch (k1) {
case "key": {
wU8(0)
r1++
const f1 = v0[k1]
if (f1 === undefined) {
return false
wU8(0)
} else if (f1 === null) {
wU8(2)
} else {
wU8(1)
if (typeof f1 !== 'string') return false
wStr(f1)
}
break
}
case "sourceId": {
wU8(1)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'string') return false
wStr(f1)
break
}
case "line": {
wU8(2)
r1++
const f1 = v0[k1]
if (f1 === undefined) {
return false
wU8(0)
} else if (f1 === null) {
wU8(2)
} else {
wU8(1)
if (typeof f1 !== 'number') return false
wF64(f1)
}
break
}
default: return false
}
if (++n1 > 255) return false
}
if (r1 !== 3) return false
iv.setUint8(at1, n1)
}
}
if (!Array.isArray(a1)) return false
wU8(1)
wU32(a1.length)
for (let i0 = 0; i0 < a1.length; i0++) {
const v0 = a1[i0]
if (v0 === null || typeof v0 !== 'object' || Array.isArray(v0)) return false
{
const at1 = at
wU8(0)
let n1 = 0, r1 = 0
for (const k1 in v0) {
switch (k1) {
case "content": {
wU8(0)
const f1 = v0[k1]
if (f1 === undefined) {
wU8(0)
} else if (f1 === null) {
wU8(2)
} else {
wU8(1)
if (typeof f1 !== 'string') return false
wStr(f1)
}
break
}
default: return false
}
if (++n1 > 255) return false
}
if (r1 !== 0) return false
iv.setUint8(at1, n1)
}
}
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.unwrittenRestartSettings = function unwrittenRestartSettings (a0, a1) {
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `completeSubscriptionUrl`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.completeSubscriptionUrl
const call = native.completeSubscriptionUrlWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.completeSubscriptionUrl = function completeSubscriptionUrl (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of completeSubscriptionUrl has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `isSubscriptionUrl`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.isSubscriptionUrl
const call = native.isSubscriptionUrlWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.isSubscriptionUrl = function isSubscriptionUrl (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of isSubscriptionUrl has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `parseInterval`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.parseInterval
const call = native.parseIntervalWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rF64()
}
}
return t
}
module.exports.parseInterval = function parseInterval (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of parseInterval has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `urlHost`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.urlHost
const call = native.urlHostWire
const encode = (a0) => {
at = 0
if (a0 === undefined) {
return false
wU8(0)
} else if (a0 === null) {
wU8(2)
} else {
wU8(1)
if (typeof a0 !== 'string') return false
wStr(a0)
}
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
t = rStr()
}
}
return t
}
module.exports.urlHost = function urlHost (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of urlHost has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `agentProblem`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.agentProblem
const call = native.agentProblemWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'boolean') return false
wU8(a1 ? 1 : 0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.agentProblem = function agentProblem (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of agentProblem has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `readSubscriptionEntries`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.readSubscriptionEntries
const call = native.readSubscriptionEntriesWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["tag"] = v1
break
}
case 1: {
let v1
v1 = rStr()
o1["url"] = v1
break
}
case 2: {
let v1
v1 = rStr()
o1["form"] = v1
break
}
case 3: {
let v1
v1 = rStr()
o1["naming"] = v1
break
}
case 4: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["ua"] = v1
break
}
case 5: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rF64()
}
}
o1["interval"] = v1
break
}
case 6: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rU8() === 1
}
}
o1["cache"] = v1
break
}
case 7: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["route"] = v1
break
}
case 8: {
let v1
if (rU8() === 0) v1 = null
else {
const n2 = rU32()
const a2 = new Array(n2)
for (let i2 = 0; i2 < n2; i2++) {
let v2
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rStr()
o3["name"] = v3
break
}
case 1: {
let v3
v3 = rStr()
o3["value"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v2 = o3
}
a2[i2] = v2
}
v1 = a2
}
o1["options"] = v1
break
}
case 9: {
let v1
v1 = rF64()
o1["from"] = v1
break
}
case 10: {
let v1
v1 = rF64()
o1["to"] = v1
break
}
case 11: {
let v1
v1 = rF64()
o1["line"] = v1
break
}
case 12: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rF64()
o3["from"] = v3
break
}
case 1: {
let v3
v3 = rF64()
o3["to"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v1 = o3
}
}
}
o1["tagAt"] = v1
break
}
case 13: {
let v1
{
const o2 = {}
const n2 = rU8()
for (let j2 = 0; j2 < n2; j2++) {
switch (rU8()) {
case 0: {
let v2
v2 = rF64()
o2["from"] = v2
break
}
case 1: {
let v2
v2 = rF64()
o2["to"] = v2
break
}
default: {
const k2 = rStr()
o2[k2] = rDyn()
}
}
}
v1 = o2
}
o1["urlAt"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.readSubscriptionEntries = function readSubscriptionEntries (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of readSubscriptionEntries has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `writeSubscriptionEntry`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.writeSubscriptionEntry
const call = native.writeSubscriptionEntryWire
const encode = (a0, a1, a2) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (typeof a1 !== 'string') return false
wStr(a1)
if (!wDyn(a2)) return false
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.writeSubscriptionEntry = function writeSubscriptionEntry (a0, a1, a2) {
  if (illFormed(a0)) throw new TypeError('argument 1 of writeSubscriptionEntry has a lone surrogate, which a Rust String cannot hold')
  if (illFormed(a1)) throw new TypeError('argument 2 of writeSubscriptionEntry has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `templateRules`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.templateRules
const call = native.templateRulesWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
v0 = rStr()
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.templateRules = function templateRules (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of templateRules has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `detectTemplate`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.detectTemplate
const call = native.detectTemplateWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rU8() === 1
o1["blockAds"] = v1
break
}
case 1: {
let v1
v1 = rU8() === 1
o1["blockQuic"] = v1
break
}
case 2: {
let v1
v1 = rU8() === 1
o1["networkManagerDirect"] = v1
break
}
case 3: {
let v1
v1 = rStr()
o1["template"] = v1
break
}
case 4: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
v1 = rStr()
}
}
o1["group"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
t = o1
}
}
}
return t
}
module.exports.detectTemplate = function detectTemplate (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of detectTemplate has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `routingOptions`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.routingOptions
const call = native.routingOptionsWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rU8() === 1
o1["blockAds"] = v1
break
}
case 1: {
let v1
v1 = rU8() === 1
o1["blockQuic"] = v1
break
}
case 2: {
let v1
v1 = rU8() === 1
o1["networkManagerDirect"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
t = o1
}
}
}
return t
}
module.exports.routingOptions = function routingOptions (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of routingOptions has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `isBareName`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.isBareName
const call = native.isBareNameWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.isBareName = function isBareName (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of isBareName has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `scanConfig`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.scanConfig
const call = native.scanConfigWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.scanConfig = function scanConfig (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of scanConfig has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `unquote`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.unquote
const call = native.unquoteWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.unquote = function unquote (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of unquote has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `blockFields`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.blockFields
const call = native.blockFieldsWire
const encode = (a0, a1, a2) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
if (!Array.isArray(a2)) return false
wU8(1)
wU32(a2.length)
for (let i0 = 0; i0 < a2.length; i0++) {
const v0 = a2[i0]
if (v0 === null || typeof v0 !== 'object' || Array.isArray(v0)) return false
{
const at1 = at
wU8(0)
let n1 = 0, r1 = 0
for (const k1 in v0) {
switch (k1) {
case "from": {
wU8(0)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'number') return false
wF64(f1)
break
}
case "to": {
wU8(1)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'number') return false
wF64(f1)
break
}
case "line": {
wU8(2)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'number') return false
wF64(f1)
break
}
case "kind": {
wU8(3)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'string') return false
wStr(f1)
break
}
case "depth": {
wU8(4)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'number') return false
wF64(f1)
break
}
case "parens": {
wU8(5)
r1++
const f1 = v0[k1]
if (typeof f1 !== 'number') return false
wF64(f1)
break
}
default: return false
}
if (++n1 > 255) return false
}
if (r1 !== 6) return false
iv.setUint8(at1, n1)
}
}
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["name"] = v1
break
}
case 1: {
let v1
v1 = rF64()
o1["from"] = v1
break
}
case 2: {
let v1
v1 = rF64()
o1["to"] = v1
break
}
case 3: {
let v1
v1 = rF64()
o1["valueFrom"] = v1
break
}
case 4: {
let v1
v1 = rF64()
o1["valueTo"] = v1
break
}
case 5: {
let v1
v1 = rStr()
o1["value"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.blockFields = function blockFields (a0, a1, a2) {
  if (illFormed(a0)) throw new TypeError('argument 1 of blockFields has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 3 && encode(a0, a1, a2) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `blockBody`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.blockBody
const call = native.blockBodyWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
v0 = rStr()
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.blockBody = function blockBody (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of blockBody has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `blockEntries`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.blockEntries
const call = native.blockEntriesWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rF64()
o1["from"] = v1
break
}
case 1: {
let v1
v1 = rF64()
o1["to"] = v1
break
}
case 2: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rStr()
o3["name"] = v3
break
}
case 1: {
let v3
v3 = rF64()
o3["from"] = v3
break
}
case 2: {
let v3
v3 = rF64()
o3["open"] = v3
break
}
case 3: {
let v3
v3 = rF64()
o3["close"] = v3
break
}
case 4: {
let v3
v3 = rF64()
o3["to"] = v3
break
}
case 5: {
let v3
v3 = rF64()
o3["line"] = v3
break
}
case 6: {
let v3
v3 = rF64()
o3["endLine"] = v3
break
}
case 7: {
let v3
v3 = rF64()
o3["depth"] = v3
break
}
case 8: {
let v3
if (rU8() === 0) v3 = null
else {
const n4 = rU32()
const a4 = new Array(n4)
for (let i4 = 0; i4 < n4; i4++) {
let v4
{
const o5 = {}
const n5 = rU8()
for (let j5 = 0; j5 < n5; j5++) {
switch (rU8()) {
case 0: {
let v5
v5 = rStr()
o5["name"] = v5
break
}
case 1: {
let v5
v5 = rF64()
o5["from"] = v5
break
}
case 2: {
let v5
v5 = rF64()
o5["open"] = v5
break
}
case 3: {
let v5
v5 = rF64()
o5["close"] = v5
break
}
case 4: {
let v5
v5 = rF64()
o5["to"] = v5
break
}
case 5: {
let v5
v5 = rF64()
o5["line"] = v5
break
}
case 6: {
let v5
v5 = rF64()
o5["endLine"] = v5
break
}
case 7: {
let v5
v5 = rF64()
o5["depth"] = v5
break
}
case 8: {
let v5
if (rU8() === 0) v5 = null
else {
const n6 = rU32()
const a6 = new Array(n6)
for (let i6 = 0; i6 < n6; i6++) {
let v6
v6 = rDyn()
a6[i6] = v6
}
v5 = a6
}
o5["children"] = v5
break
}
default: {
const k5 = rStr()
o5[k5] = rDyn()
}
}
}
v4 = o5
}
a4[i4] = v4
}
v3 = a4
}
o3["children"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v1 = o3
}
}
}
o1["block"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
v0 = o1
}
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.blockEntries = function blockEntries (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of blockEntries has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `policyKind`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.policyKind
const call = native.policyKindWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
t = rDyn()
return t
}
module.exports.policyKind = function policyKind (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of policyKind has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `readXhttp`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.readXhttp
const call = native.readXhttpWire
const encode = (a0) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
return true
}
const decode = () => {
let t
{
const g0 = rU8()
if (g0 === 0) t = undefined
else if (g0 === 2) t = null
else {
{
const o1 = {}
const n1 = rU8()
for (let j1 = 0; j1 < n1; j1++) {
switch (rU8()) {
case 0: {
let v1
v1 = rStr()
o1["mode"] = v1
break
}
case 1: {
let v1
v1 = rStr()
o1["path"] = v1
break
}
case 2: {
let v1
v1 = rStr()
o1["host"] = v1
break
}
case 3: {
let v1
if (rU8() === 0) v1 = null
else {
const n2 = rU32()
const a2 = new Array(n2)
for (let i2 = 0; i2 < n2; i2++) {
let v2
v2 = rDyn()
a2[i2] = v2
}
v1 = a2
}
o1["headers"] = v1
break
}
case 4: {
let v1
v1 = rStr()
o1["xPaddingBytes"] = v1
break
}
case 5: {
let v1
v1 = rU8() === 1
o1["noGRPCHeader"] = v1
break
}
case 6: {
let v1
v1 = rStr()
o1["scMaxEachPostBytes"] = v1
break
}
case 7: {
let v1
v1 = rStr()
o1["scMinPostsIntervalMs"] = v1
break
}
case 8: {
let v1
{
const g2 = rU8()
if (g2 === 0) v1 = undefined
else if (g2 === 2) v1 = null
else {
{
const o3 = {}
const n3 = rU8()
for (let j3 = 0; j3 < n3; j3++) {
switch (rU8()) {
case 0: {
let v3
v3 = rStr()
o3["address"] = v3
break
}
case 1: {
let v3
v3 = rStr()
o3["port"] = v3
break
}
case 2: {
let v3
v3 = rStr()
o3["serverName"] = v3
break
}
case 3: {
let v3
v3 = rStr()
o3["path"] = v3
break
}
case 4: {
let v3
v3 = rStr()
o3["host"] = v3
break
}
default: {
const k3 = rStr()
o3[k3] = rDyn()
}
}
}
v1 = o3
}
}
}
o1["download"] = v1
break
}
default: {
const k1 = rStr()
o1[k1] = rDyn()
}
}
}
t = o1
}
}
}
return t
}
module.exports.readXhttp = function readXhttp (a0) {
  if (illFormed(a0)) throw new TypeError('argument 1 of readXhttp has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `duplicateXhttpHeaders`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.duplicateXhttpHeaders
const call = native.duplicateXhttpHeadersWire
const encode = (a0) => {
at = 0
if (!wDyn(a0)) return false
return true
}
const decode = () => {
let t
if (rU8() === 0) t = null
else {
const n0 = rU32()
const a0 = new Array(n0)
for (let i0 = 0; i0 < n0; i0++) {
let v0
v0 = rF64()
a0[i0] = v0
}
t = a0
}
return t
}
module.exports.duplicateXhttpHeaders = function duplicateXhttpHeaders (a0) {
  try {
    if (arguments.length <= 1 && encode(a0) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}

// `writeXhttp`: binary wire; the JSON version handles inputs of other shapes.
{
const json = module.exports.writeXhttp
const call = native.writeXhttpWire
const encode = (a0, a1) => {
at = 0
if (typeof a0 !== 'string') return false
wStr(a0)
if (!wDyn(a1)) return false
return true
}
const decode = () => {
let t
t = rStr()
return t
}
module.exports.writeXhttp = function writeXhttp (a0, a1) {
  if (illFormed(a0)) throw new TypeError('argument 1 of writeXhttp has a lone surrogate, which a Rust String cannot hold')
  try {
    if (arguments.length <= 2 && encode(a0, a1) && wireCall(call)) return decode()
    return json(...arguments)
  } catch (e) {
    throw jsError(e)
  }
}
}
const guards = {
  validGeodataUrl: [false, [0]],
  splitTop: [false, [0, 1]],
  includePatterns: [false, [0]],
  globMatch: [false, [0, 1]],
  includeDirectory: [false, [0]],
  isProtocol: [false, [0]],
  readRuleFallback: [false, [0]],
  writeRuleFallback: [false, [0, 1]],
  holdsDns: [false, [0]],
  holdsRouting: [false, [0]],
  templateGroups: [false, [0, 1]],
  isPresetRouting: [false, [0]],
  quote: [false, [0]],
  isQuotable: [false, [0]],
  isFragment: [false, [0]],
  uncomment: [false, [0]],
  isBuiltinOutbound: [false, [0]],
}
const JS_ERRORS = /^(TypeError|RangeError|SyntaxError|ReferenceError|URIError|EvalError): ([\s\S]*)$/
const jsError = e => {
  const message = typeof e?.message === 'string' ? e.message : ''
  const typed = JS_ERRORS.exec(message)
  if (typed) return new globalThis[typed[1]](typed[2])
  if (/Expected$/.test(e?.code)) return Object.assign(new TypeError(message), { code: e.code })
  if (message.startsWith('Rust panic:')) e.code = 'RUST_PANIC'
  return e
}
const illFormed = v => typeof v === 'string' ? !v.isWellFormed() : Array.isArray(v) && v.some(illFormed)
for (const [name, [isAsync, strings]] of Object.entries(guards)) {
  const f = module.exports[name]
  if (typeof f !== 'function') continue
  const check = a => {
    for (const i of strings) {
      if (illFormed(a[i])) throw new TypeError(`argument ${i + 1} of ${name} has a lone surrogate, which a Rust String cannot hold`)
    }
  }
  module.exports[name] = isAsync
    ? { [name]: async function (...a) { check(a); try { return await f(...a) } catch (e) { throw jsError(e) } } }[name]
    : { [name]: function (...a) { check(a); try { return f(...a) } catch (e) { throw jsError(e) } } }[name]
}

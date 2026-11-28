package io.honk.android.tile

import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import io.honk.android.bridge.BoxEngineHolder
import java.util.concurrent.Executors

/**
 * M55 Ultra-thin Quick Settings Tile delegating 100% of state and toggle logic to Rust.
 */
class BoxServiceTileService : TileService() {
    private val ioExecutor = Executors.newSingleThreadExecutor()

    override fun onStartListening() {
        super.onStartListening()
        ioExecutor.execute {
            BoxEngineHolder.dispatchAction("refresh_status")
            updateTileUi()
        }
    }

    override fun onClick() {
        super.onClick()
        unlockAndRun {
            ioExecutor.execute {
                BoxEngineHolder.dispatchAction("tile_toggle")
                updateTileUi()
            }
        }
    }

    private fun updateTileUi() {
        val snap = BoxEngineHolder.queryTileState()
        val active = snap.optBoolean("active", false)
        val working = snap.optBoolean("working", false)
        val label = snap.optString("label", "Box Honk")
        val subtitle = snap.optString("subtitle", "")

        qsTile?.let { tile ->
            tile.state = when {
                working -> Tile.STATE_UNAVAILABLE
                active -> Tile.STATE_ACTIVE
                else -> Tile.STATE_INACTIVE
            }
            tile.label = label
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                tile.subtitle = subtitle
            }
            tile.updateTile()
        }
    }
}

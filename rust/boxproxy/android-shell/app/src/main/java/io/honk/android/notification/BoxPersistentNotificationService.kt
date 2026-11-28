package io.honk.android.notification

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import io.honk.android.MainActivity
import io.honk.android.PanelSheetActivity
import io.honk.android.R
import io.honk.android.bridge.BoxEngineHolder
import org.json.JSONObject
import java.util.concurrent.Executors

/**
 * M57 Ultra-thin Android 14+ `specialUse` persistent status notification service.
 * 12-placeholder rendering, button labels, and visibility policy are 100% decided by Rust.
 */
class BoxPersistentNotificationService : Service() {
    companion object {
        const val CHANNEL_ID = "box_persistent_status"
        const val NOTIFICATION_ID = 1001
        const val ACTION_RELOAD = "io.honk.android.notification.ACTION_RELOAD"
        const val ACTION_RESTART = "io.honk.android.notification.ACTION_RESTART"
        const val ACTION_STOP = "io.honk.android.notification.ACTION_STOP"
    }

    private val handler = Handler(Looper.getMainLooper())
    private val worker = Executors.newSingleThreadExecutor()
    @Volatile
    private var running = false

    private val refreshTick: Runnable = object : Runnable {
        override fun run() {
            if (!running) return
            worker.execute {
                val snap = BoxEngineHolder.renderNotification(forceRefresh = true)
                handler.post {
                    if (!snap.optBoolean("should_show", false)) {
                        stopForeground(STOP_FOREGROUND_REMOVE)
                        stopSelf()
                        return@post
                    }
                    postNotification(snap)
                    val delayMs = snap.optInt("refresh_interval_seconds", 5).coerceAtLeast(1) * 1000L
                    handler.postDelayed(this@BoxPersistentNotificationService.refreshTick, delayMs)
                }
            }
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        ensureChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_RELOAD -> worker.execute { BoxEngineHolder.dispatchAction("service_reload") }
            ACTION_RESTART -> worker.execute { BoxEngineHolder.dispatchAction("service_restart") }
            ACTION_STOP -> worker.execute { BoxEngineHolder.dispatchAction("service_down") }
        }
        val initial = BoxEngineHolder.renderNotification(forceRefresh = false)
        val notif = buildNotification(initial)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            startForeground(NOTIFICATION_ID, notif, ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE)
        } else {
            startForeground(NOTIFICATION_ID, notif)
        }
        if (!running) {
            running = true
            handler.postDelayed(refreshTick, 1000L)
        }
        return START_STICKY
    }

    override fun onDestroy() {
        running = false
        handler.removeCallbacks(refreshTick)
        worker.shutdownNow()
        super.onDestroy()
    }

    private fun ensureChannel() {
        val nm = getSystemService(NotificationManager::class.java) ?: return
        val channel = NotificationChannel(
            CHANNEL_ID,
            getString(R.string.notification_channel_name),
            NotificationManager.IMPORTANCE_LOW,
        ).apply {
            setSound(null, null)
            enableVibration(false)
            setShowBadge(false)
        }
        nm.createNotificationChannel(channel)
    }

    private fun postNotification(snap: JSONObject) {
        val nm = getSystemService(NotificationManager::class.java) ?: return
        nm.notify(NOTIFICATION_ID, buildNotification(snap))
    }

    private fun buildNotification(snap: JSONObject): Notification {
        val title = snap.optString("title", "BoxProxy Honk")
        val content = snap.optString("content", "")
        val clickTarget = snap.optString("click_target", "panel")

        val clickIntent = when (clickTarget) {
            "panel" -> Intent(this, PanelSheetActivity::class.java)
            "web_ui" -> Intent(this, MainActivity::class.java).putExtra(MainActivity.EXTRA_TARGET, "web_ui")
            else -> Intent(this, MainActivity::class.java).putExtra(MainActivity.EXTRA_TARGET, "app")
        }.apply {
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
        }

        val contentPi = PendingIntent.getActivity(
            this,
            100,
            clickIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )

        val builder = Notification.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_stat_box)
            .setContentTitle(title)
            .setContentText(content.lineSequence().firstOrNull().orEmpty())
            .setStyle(Notification.BigTextStyle().bigText(content))
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setContentIntent(contentPi)

        val extras = android.os.Bundle().apply { putBoolean("android.app.preferSmallIcon", true) }
        builder.addExtras(extras)

        val buttons = snap.optJSONArray("buttons")
        if (buttons != null) {
            for (i in 0 until buttons.length().coerceAtMost(3)) {
                val btn = buttons.optJSONObject(i) ?: continue
                val target = btn.optString("target")
                val label = btn.optString("label", target)
                val pi = buildActionPendingIntent(101 + i, target) ?: continue
                builder.addAction(Notification.Action.Builder(null, label, pi).build())
            }
        }
        return builder.build()
    }

    private fun buildActionPendingIntent(requestCode: Int, target: String): PendingIntent? {
        val flags = PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        return when (target) {
            "reload" -> PendingIntent.getService(
                this,
                requestCode,
                Intent(this, BoxPersistentNotificationService::class.java).setAction(ACTION_RELOAD),
                flags,
            )
            "restart" -> PendingIntent.getService(
                this,
                requestCode,
                Intent(this, BoxPersistentNotificationService::class.java).setAction(ACTION_RESTART),
                flags,
            )
            "stop" -> PendingIntent.getService(
                this,
                requestCode,
                Intent(this, BoxPersistentNotificationService::class.java).setAction(ACTION_STOP),
                flags,
            )
            "open_panel" -> PendingIntent.getActivity(
                this,
                requestCode,
                Intent(this, PanelSheetActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
                flags,
            )
            "open_web_ui" -> PendingIntent.getActivity(
                this,
                requestCode,
                Intent(this, MainActivity::class.java)
                    .putExtra(MainActivity.EXTRA_TARGET, "web_ui")
                    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
                flags,
            )
            else -> null
        }
    }
}

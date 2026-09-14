package org.vocaltract.pixel;

import android.app.Application;
import android.os.Process;
import java.lang.Thread;
import kotlin.Metadata;
import kotlin.Result;
import kotlin.ResultKt;
import kotlin.TuplesKt;
import kotlin.Unit;
import kotlin.collections.ArraysKt;
import kotlin.collections.CollectionsKt;
import kotlin.collections.MapsKt;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: VocalTractApplication.kt */
@Metadata(d1 = {"\u0000\u0010\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0010\u0002\u0018\u00002\u00020\u0001B\u0007¢\u0006\u0004\b\u0002\u0010\u0003J\b\u0010\u0004\u001a\u00020\u0005H\u0016"}, d2 = {"Lorg/vocaltract/pixel/VocalTractApplication;", "Landroid/app/Application;", "<init>", "()V", "onCreate", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class VocalTractApplication extends Application {
    @Override // android.app.Application
    public void onCreate() {
        super.onCreate();
        DiagnosticsRuntime.INSTANCE.initialize(this);
        final Thread.UncaughtExceptionHandler defaultUncaughtExceptionHandler = Thread.getDefaultUncaughtExceptionHandler();
        Thread.setDefaultUncaughtExceptionHandler(new Thread.UncaughtExceptionHandler() { // from class: org.vocaltract.pixel.VocalTractApplication$$ExternalSyntheticLambda0
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Thread.UncaughtExceptionHandler
            public final void uncaughtException(Thread thread, Throwable th) {
                VocalTractApplication.onCreate$lambda$1(VocalTractApplication.this, defaultUncaughtExceptionHandler, thread, th);
            }
        });
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void onCreate$lambda$1(VocalTractApplication vocalTractApplication, Thread.UncaughtExceptionHandler uncaughtExceptionHandler, Thread thread, Throwable th) {
        try {
            Result.Companion companion = Result.INSTANCE;
            DiagnosticsRuntime diagnosticsRuntime = DiagnosticsRuntime.INSTANCE;
            DiagnosticSeverity diagnosticSeverity = DiagnosticSeverity.ERROR;
            String simpleName = th.getClass().getSimpleName();
            String message = th.getMessage();
            if (message == null) {
                message = "no message";
            }
            StackTraceElement[] stackTrace = th.getStackTrace();
            Intrinsics.checkNotNullExpressionValue(stackTrace, "getStackTrace(...)");
            diagnosticsRuntime.log("crash", "uncaught_exception", simpleName + ": " + message, MapsKt.mapOf(TuplesKt.to("thread", thread.getName()), TuplesKt.to("top_frames", CollectionsKt.joinToString$default(ArraysKt.take(stackTrace, 8), " | ", null, null, 0, null, null, 62, null))), diagnosticSeverity);
            Result.m4constructorimpl(Unit.INSTANCE);
        } catch (Throwable th2) {
            Result.Companion companion2 = Result.INSTANCE;
            Result.m4constructorimpl(ResultKt.createFailure(th2));
        }
        if (uncaughtExceptionHandler != null) {
            uncaughtExceptionHandler.uncaughtException(thread, th);
        } else {
            Process.killProcess(Process.myPid());
            System.exit(10);
            throw new RuntimeException("System.exit returned normally, while it was supposed to halt JVM.");
        }
    }
}

package org.vocaltract.pixel;

import android.content.Context;
import android.content.SharedPreferences;
import android.media.AudioRecord;
import java.io.File;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicLong;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.UnaryOperator;
import kotlin.Metadata;
import kotlin.Pair;
import kotlin.Result;
import kotlin.ResultKt;
import kotlin.TuplesKt;
import kotlin.collections.ArraysKt;
import kotlin.collections.CollectionsKt;
import kotlin.collections.MapsKt;
import kotlin.io.ConstantsKt;
import kotlin.jvm.functions.Function0;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.Intrinsics;
import kotlin.text.StringsKt;
import kotlin.uuid.Uuid;
import org.json.JSONObject;
import org.vocaltract.pixel.AdditiveSynthState;
import org.vocaltract.pixel.DiagnosticAssistant;
import org.vocaltract.pixel.DiagnosticStore;

/* compiled from: DiagnosticsRuntime.kt */
@Metadata(d1 = {"\u0000¦\u0001\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010 \n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u000e\n\u0002\b\u0007\n\u0002\u0010\u0002\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\b\n\u0002\b\u0004\n\u0002\u0010\u000b\n\u0002\b\u0004\n\u0002\u0018\u0002\n\u0002\b\u0005\n\u0002\u0010$\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0004\n\u0002\u0018\u0002\n\u0002\b\u0005\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0010\t\bÆ\u0002\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u000e\u0010\u001e\u001a\u00020\u001f2\u0006\u0010 \u001a\u00020\u0010J\u0006\u0010!\u001a\u00020\bJ\u0006\u0010\"\u001a\u00020#J\b\u0010$\u001a\u0004\u0018\u00010\u0015J\u0016\u0010%\u001a\b\u0012\u0004\u0012\u00020&0\u00122\b\b\u0002\u0010'\u001a\u00020(J\u000e\u0010)\u001a\u00020\u001f2\u0006\u0010*\u001a\u00020\u0017J\u000e\u0010+\u001a\u00020\u001f2\u0006\u0010,\u001a\u00020-J\u000e\u0010.\u001a\u00020\u001f2\u0006\u0010/\u001a\u00020-J\u000e\u00100\u001a\u00020\u001f2\u0006\u00101\u001a\u000202J>\u00103\u001a\u00020\u001f2\u0006\u00104\u001a\u00020\u00172\u0006\u00105\u001a\u00020\u00172\u0006\u00106\u001a\u00020\u00172\u0014\b\u0002\u00107\u001a\u000e\u0012\u0004\u0012\u00020\u0017\u0012\u0004\u0012\u00020\u0017082\b\b\u0002\u00109\u001a\u00020:J\u000e\u0010;\u001a\u00020<2\u0006\u0010=\u001a\u00020>J\u000e\u0010?\u001a\u00020\u001f2\u0006\u0010@\u001a\u00020\u0015J\u0014\u0010A\u001a\b\u0012\u0004\u0012\u00020\u00130\u00122\u0006\u0010 \u001a\u00020\u0010J\u0006\u0010B\u001a\u00020CJ\u0006\u0010D\u001a\u00020\u001fJ\u000e\u0010E\u001a\u00020-2\u0006\u0010 \u001a\u00020\u0010J\u0016\u0010F\u001a\u00020\u001f2\u0006\u0010 \u001a\u00020\u00102\u0006\u0010G\u001a\u00020-J\u001d\u0010H\u001a\n \t*\u0004\u0018\u00010I0I2\u0006\u0010 \u001a\u00020\u0010H\u0002¢\u0006\u0002\u0010JR\u000e\u0010\u0004\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u001c\u0010\u0006\u001a\u0010\u0012\f\u0012\n \t*\u0004\u0018\u00010\b0\b0\u0007X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\n\u001a\u00020\u000bX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\f\u001a\u00020\u000bX\u0082\u0004¢\u0006\u0002\n\u0000R\u0010\u0010\r\u001a\u0004\u0018\u00010\u000eX\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010\u000f\u001a\u0004\u0018\u00010\u0010X\u0082\u000e¢\u0006\u0002\n\u0000R\u0014\u0010\u0011\u001a\b\u0012\u0004\u0012\u00020\u00130\u0012X\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010\u0014\u001a\u0004\u0018\u00010\u0015X\u0082\u000e¢\u0006\u0002\n\u0000R\u001c\u0010\u0016\u001a\u0004\u0018\u00010\u0017X\u0086\u000e¢\u0006\u000e\n\u0000\u001a\u0004\b\u0018\u0010\u0019\"\u0004\b\u001a\u0010\u001bR\u0011\u0010\u001c\u001a\u00020\u00178F¢\u0006\u0006\u001a\u0004\b\u001d\u0010\u0019R\u000e\u0010K\u001a\u00020\u0017X\u0082T¢\u0006\u0002\n\u0000R\u000e\u0010L\u001a\u00020MX\u0082T¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/DiagnosticsRuntime;", "", "<init>", "()V", "assistant", "Lorg/vocaltract/pixel/DiagnosticAssistant;", "metrics", "Ljava/util/concurrent/atomic/AtomicReference;", "Lorg/vocaltract/pixel/DiagnosticMetrics;", "kotlin.jvm.PlatformType", "lastSampleMillis", "Ljava/util/concurrent/atomic/AtomicLong;", "lastLoggedDrops", "store", "Lorg/vocaltract/pixel/DiagnosticStore;", "applicationContext", "Landroid/content/Context;", "selfTests", "", "Lorg/vocaltract/pixel/SelfTestResult;", "calibrationReport", "Lorg/vocaltract/pixel/SingerCalibrationReport;", "sharedModelSnapshot", "", "getSharedModelSnapshot", "()Ljava/lang/String;", "setSharedModelSnapshot", "(Ljava/lang/String;)V", "sessionId", "getSessionId", "initialize", "", "context", "currentMetrics", "currentAssessment", "Lorg/vocaltract/pixel/DiagnosticAssessment;", "currentCalibrationReport", "recentEvents", "Lorg/vocaltract/pixel/DiagnosticEvent;", "limit", "", "updateRendererMode", "mode", "updateMicrophoneGranted", "granted", "", "updateSynthesizerActive", "active", "updateState", "state", "Lorg/vocaltract/pixel/VocalAcousticsState;", "log", "category", "code", "message", "evidence", "", "severity", "Lorg/vocaltract/pixel/DiagnosticSeverity;", "recordRefinement", "Lorg/vocaltract/pixel/RefinementRecord;", "draft", "Lorg/vocaltract/pixel/RefinementDraft;", "recordCalibration", "report", "runSelfTests", "exportBundle", "Lorg/vocaltract/pixel/DiagnosticExport;", "clearLogs", "debugOverlayEnabled", "setDebugOverlayEnabled", "enabled", "preferences", "Landroid/content/SharedPreferences;", "(Landroid/content/Context;)Landroid/content/SharedPreferences;", "OVERLAY_KEY", "SAMPLE_INTERVAL_MILLIS", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class DiagnosticsRuntime {
    private static final String OVERLAY_KEY = "show_debug_overlay";
    private static final long SAMPLE_INTERVAL_MILLIS = 500;
    private static volatile Context applicationContext;
    private static volatile SingerCalibrationReport calibrationReport;
    private static volatile String sharedModelSnapshot;
    private static volatile DiagnosticStore store;
    public static final DiagnosticsRuntime INSTANCE = new DiagnosticsRuntime();
    private static final DiagnosticAssistant assistant = new OfflineDiagnosticAssistant();
    private static final AtomicReference<DiagnosticMetrics> metrics = new AtomicReference<>(new DiagnosticMetrics(0, null, 0.0f, 0.0f, 0, false, null, 0.0f, 0.0f, 0.0f, 0.0f, null, false, false, null, null, false, 0.0f, 0.0f, 0.0f, null, 0.0f, false, null, false, null, null, null, 0.0f, 0.0f, 1073741823, null));
    private static final AtomicLong lastSampleMillis = new AtomicLong(0);
    private static final AtomicLong lastLoggedDrops = new AtomicLong(-1);
    private static volatile List<SelfTestResult> selfTests = CollectionsKt.emptyList();

    private DiagnosticsRuntime() {
    }

    public final String getSharedModelSnapshot() {
        return sharedModelSnapshot;
    }

    public final void setSharedModelSnapshot(String str) {
        sharedModelSnapshot = str;
    }

    public final String getSessionId() {
        String sessionId;
        DiagnosticStore diagnosticStore = store;
        return (diagnosticStore == null || (sessionId = diagnosticStore.getSessionId()) == null) ? "not-initialized" : sessionId;
    }

    /* JADX DEBUG: Class process forced to load method for inline: org.vocaltract.pixel.DiagnosticStore.Companion.newSessionId$default(org.vocaltract.pixel.DiagnosticStore$Companion, long, int, java.lang.Object):java.lang.String */
    public final synchronized void initialize(Context context) {
        Intrinsics.checkNotNullParameter(context, "context");
        if (store != null) {
            return;
        }
        applicationContext = context.getApplicationContext();
        Context applicationContext2 = context.getApplicationContext();
        Intrinsics.checkNotNullExpressionValue(applicationContext2, "getApplicationContext(...)");
        store = new DiagnosticStore(applicationContext2, DiagnosticStore.Companion.newSessionId$default(DiagnosticStore.INSTANCE, 0L, 1, null));
        DiagnosticStore diagnosticStore = store;
        DiagnosticEvent consumePendingCrash = diagnosticStore != null ? diagnosticStore.consumePendingCrash() : null;
        log$default(this, "lifecycle", "process_started", "Diagnostics session started", MapsKt.mapOf(TuplesKt.to("privacy", "derived_metrics_only"), TuplesKt.to("network", "disabled")), null, 16, null);
        if (consumePendingCrash != null) {
            log("recovery", "previous_crash_recovered", consumePendingCrash.getMessage(), MapsKt.plus(consumePendingCrash.getEvidence(), MapsKt.mapOf(TuplesKt.to("previous_session", consumePendingCrash.getSessionId()), TuplesKt.to("crash_timestamp_epoch_ms", String.valueOf(consumePendingCrash.getTimestampMillis())))), DiagnosticSeverity.ERROR);
        }
    }

    public final DiagnosticMetrics currentMetrics() {
        DiagnosticMetrics diagnosticMetrics = metrics.get();
        Intrinsics.checkNotNullExpressionValue(diagnosticMetrics, "get(...)");
        return diagnosticMetrics;
    }

    public final DiagnosticAssessment currentAssessment() {
        DiagnosticAssistant diagnosticAssistant = assistant;
        DiagnosticMetrics diagnosticMetrics = metrics.get();
        Intrinsics.checkNotNullExpressionValue(diagnosticMetrics, "get(...)");
        return DiagnosticAssistant.DefaultImpls.assess$default(diagnosticAssistant, diagnosticMetrics, 0L, 2, null);
    }

    public final SingerCalibrationReport currentCalibrationReport() {
        return calibrationReport;
    }

    public static /* synthetic */ List recentEvents$default(DiagnosticsRuntime diagnosticsRuntime, int i, int i2, Object obj) {
        if ((i2 & 1) != 0) {
            i = 60;
        }
        return diagnosticsRuntime.recentEvents(i);
    }

    public final List<DiagnosticEvent> recentEvents(int limit) {
        DiagnosticStore diagnosticStore = store;
        List<DiagnosticEvent> recentEvents = diagnosticStore != null ? diagnosticStore.recentEvents(limit) : null;
        return recentEvents == null ? CollectionsKt.emptyList() : recentEvents;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final DiagnosticMetrics updateRendererMode$lambda$0(String str, DiagnosticMetrics diagnosticMetrics) {
        DiagnosticMetrics copy;
        copy = diagnosticMetrics.copy((r50 & 1) != 0 ? diagnosticMetrics.sequence : 0L, (r50 & 2) != 0 ? diagnosticMetrics.source : null, (r50 & 4) != 0 ? diagnosticMetrics.processingMs : 0.0f, (r50 & 8) != 0 ? diagnosticMetrics.frameBudgetMs : 0.0f, (r50 & 16) != 0 ? diagnosticMetrics.droppedFrames : 0L, (r50 & 32) != 0 ? diagnosticMetrics.voiced : false, (r50 & 64) != 0 ? diagnosticMetrics.f0Hz : null, (r50 & Uuid.SIZE_BITS) != 0 ? diagnosticMetrics.f0Confidence : 0.0f, (r50 & 256) != 0 ? diagnosticMetrics.meanFormantStdHz : 0.0f, (r50 & ConstantsKt.MINIMUM_BLOCK_SIZE) != 0 ? diagnosticMetrics.tractConfidence : 0.0f, (r50 & 1024) != 0 ? diagnosticMetrics.relativeAreaStd : 0.0f, (r50 & 2048) != 0 ? diagnosticMetrics.rendererMode : str, (r50 & ConstantsKt.DEFAULT_BLOCK_SIZE) != 0 ? diagnosticMetrics.microphoneGranted : false, (r50 & ConstantsKt.DEFAULT_BUFFER_SIZE) != 0 ? diagnosticMetrics.synthesizerActive : false, (r50 & 16384) != 0 ? diagnosticMetrics.rawF0Hz : null, (r50 & 32768) != 0 ? diagnosticMetrics.pitchDecision : null, (r50 & 65536) != 0 ? diagnosticMetrics.pitchRejected : false, (r50 & 131072) != 0 ? diagnosticMetrics.harmonicity : 0.0f, (r50 & 262144) != 0 ? diagnosticMetrics.snrDb : 0.0f, (r50 & 524288) != 0 ? diagnosticMetrics.noiseFloorDb : 0.0f, (r50 & 1048576) != 0 ? diagnosticMetrics.noiseState : null, (r50 & 2097152) != 0 ? diagnosticMetrics.noiseConfidence : 0.0f, (r50 & 4194304) != 0 ? diagnosticMetrics.backgroundChanged : false, (r50 & 8388608) != 0 ? diagnosticMetrics.noiseBandsDb : null, (r50 & 16777216) != 0 ? diagnosticMetrics.posteriorAbstained : false, (r50 & 33554432) != 0 ? diagnosticMetrics.abstentionReason : null, (r50 & 67108864) != 0 ? diagnosticMetrics.formantCandidatesHz : null, (r50 & 134217728) != 0 ? diagnosticMetrics.articulators : null, (r50 & 268435456) != 0 ? diagnosticMetrics.analysisWindowMs : 0.0f, (r50 & 536870912) != 0 ? diagnosticMetrics.analysisHopMs : 0.0f);
        return copy;
    }

    public final void updateRendererMode(final String mode) {
        Intrinsics.checkNotNullParameter(mode, "mode");
        metrics.updateAndGet(new UnaryOperator() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda10
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.util.function.Function
            public final Object apply(Object obj) {
                DiagnosticMetrics updateRendererMode$lambda$0;
                updateRendererMode$lambda$0 = DiagnosticsRuntime.updateRendererMode$lambda$0(mode, (DiagnosticMetrics) obj);
                return updateRendererMode$lambda$0;
            }
        });
        log$default(this, "renderer", "renderer_mode", "Renderer mode changed", MapsKt.mapOf(TuplesKt.to("mode", mode)), null, 16, null);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final DiagnosticMetrics updateMicrophoneGranted$lambda$1(boolean z, DiagnosticMetrics diagnosticMetrics) {
        DiagnosticMetrics copy;
        copy = diagnosticMetrics.copy((r50 & 1) != 0 ? diagnosticMetrics.sequence : 0L, (r50 & 2) != 0 ? diagnosticMetrics.source : null, (r50 & 4) != 0 ? diagnosticMetrics.processingMs : 0.0f, (r50 & 8) != 0 ? diagnosticMetrics.frameBudgetMs : 0.0f, (r50 & 16) != 0 ? diagnosticMetrics.droppedFrames : 0L, (r50 & 32) != 0 ? diagnosticMetrics.voiced : false, (r50 & 64) != 0 ? diagnosticMetrics.f0Hz : null, (r50 & Uuid.SIZE_BITS) != 0 ? diagnosticMetrics.f0Confidence : 0.0f, (r50 & 256) != 0 ? diagnosticMetrics.meanFormantStdHz : 0.0f, (r50 & ConstantsKt.MINIMUM_BLOCK_SIZE) != 0 ? diagnosticMetrics.tractConfidence : 0.0f, (r50 & 1024) != 0 ? diagnosticMetrics.relativeAreaStd : 0.0f, (r50 & 2048) != 0 ? diagnosticMetrics.rendererMode : null, (r50 & ConstantsKt.DEFAULT_BLOCK_SIZE) != 0 ? diagnosticMetrics.microphoneGranted : z, (r50 & ConstantsKt.DEFAULT_BUFFER_SIZE) != 0 ? diagnosticMetrics.synthesizerActive : false, (r50 & 16384) != 0 ? diagnosticMetrics.rawF0Hz : null, (r50 & 32768) != 0 ? diagnosticMetrics.pitchDecision : null, (r50 & 65536) != 0 ? diagnosticMetrics.pitchRejected : false, (r50 & 131072) != 0 ? diagnosticMetrics.harmonicity : 0.0f, (r50 & 262144) != 0 ? diagnosticMetrics.snrDb : 0.0f, (r50 & 524288) != 0 ? diagnosticMetrics.noiseFloorDb : 0.0f, (r50 & 1048576) != 0 ? diagnosticMetrics.noiseState : null, (r50 & 2097152) != 0 ? diagnosticMetrics.noiseConfidence : 0.0f, (r50 & 4194304) != 0 ? diagnosticMetrics.backgroundChanged : false, (r50 & 8388608) != 0 ? diagnosticMetrics.noiseBandsDb : null, (r50 & 16777216) != 0 ? diagnosticMetrics.posteriorAbstained : false, (r50 & 33554432) != 0 ? diagnosticMetrics.abstentionReason : null, (r50 & 67108864) != 0 ? diagnosticMetrics.formantCandidatesHz : null, (r50 & 134217728) != 0 ? diagnosticMetrics.articulators : null, (r50 & 268435456) != 0 ? diagnosticMetrics.analysisWindowMs : 0.0f, (r50 & 536870912) != 0 ? diagnosticMetrics.analysisHopMs : 0.0f);
        return copy;
    }

    public final void updateMicrophoneGranted(final boolean granted) {
        metrics.updateAndGet(new UnaryOperator() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda13
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.util.function.Function
            public final Object apply(Object obj) {
                DiagnosticMetrics updateMicrophoneGranted$lambda$1;
                updateMicrophoneGranted$lambda$1 = DiagnosticsRuntime.updateMicrophoneGranted$lambda$1(granted, (DiagnosticMetrics) obj);
                return updateMicrophoneGranted$lambda$1;
            }
        });
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final DiagnosticMetrics updateSynthesizerActive$lambda$2(boolean z, DiagnosticMetrics diagnosticMetrics) {
        DiagnosticMetrics copy;
        copy = diagnosticMetrics.copy((r50 & 1) != 0 ? diagnosticMetrics.sequence : 0L, (r50 & 2) != 0 ? diagnosticMetrics.source : null, (r50 & 4) != 0 ? diagnosticMetrics.processingMs : 0.0f, (r50 & 8) != 0 ? diagnosticMetrics.frameBudgetMs : 0.0f, (r50 & 16) != 0 ? diagnosticMetrics.droppedFrames : 0L, (r50 & 32) != 0 ? diagnosticMetrics.voiced : false, (r50 & 64) != 0 ? diagnosticMetrics.f0Hz : null, (r50 & Uuid.SIZE_BITS) != 0 ? diagnosticMetrics.f0Confidence : 0.0f, (r50 & 256) != 0 ? diagnosticMetrics.meanFormantStdHz : 0.0f, (r50 & ConstantsKt.MINIMUM_BLOCK_SIZE) != 0 ? diagnosticMetrics.tractConfidence : 0.0f, (r50 & 1024) != 0 ? diagnosticMetrics.relativeAreaStd : 0.0f, (r50 & 2048) != 0 ? diagnosticMetrics.rendererMode : null, (r50 & ConstantsKt.DEFAULT_BLOCK_SIZE) != 0 ? diagnosticMetrics.microphoneGranted : false, (r50 & ConstantsKt.DEFAULT_BUFFER_SIZE) != 0 ? diagnosticMetrics.synthesizerActive : z, (r50 & 16384) != 0 ? diagnosticMetrics.rawF0Hz : null, (r50 & 32768) != 0 ? diagnosticMetrics.pitchDecision : null, (r50 & 65536) != 0 ? diagnosticMetrics.pitchRejected : false, (r50 & 131072) != 0 ? diagnosticMetrics.harmonicity : 0.0f, (r50 & 262144) != 0 ? diagnosticMetrics.snrDb : 0.0f, (r50 & 524288) != 0 ? diagnosticMetrics.noiseFloorDb : 0.0f, (r50 & 1048576) != 0 ? diagnosticMetrics.noiseState : null, (r50 & 2097152) != 0 ? diagnosticMetrics.noiseConfidence : 0.0f, (r50 & 4194304) != 0 ? diagnosticMetrics.backgroundChanged : false, (r50 & 8388608) != 0 ? diagnosticMetrics.noiseBandsDb : null, (r50 & 16777216) != 0 ? diagnosticMetrics.posteriorAbstained : false, (r50 & 33554432) != 0 ? diagnosticMetrics.abstentionReason : null, (r50 & 67108864) != 0 ? diagnosticMetrics.formantCandidatesHz : null, (r50 & 134217728) != 0 ? diagnosticMetrics.articulators : null, (r50 & 268435456) != 0 ? diagnosticMetrics.analysisWindowMs : 0.0f, (r50 & 536870912) != 0 ? diagnosticMetrics.analysisHopMs : 0.0f);
        return copy;
    }

    public final void updateSynthesizerActive(final boolean active) {
        metrics.updateAndGet(new UnaryOperator() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda12
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.util.function.Function
            public final Object apply(Object obj) {
                DiagnosticMetrics updateSynthesizerActive$lambda$2;
                updateSynthesizerActive$lambda$2 = DiagnosticsRuntime.updateSynthesizerActive$lambda$2(active, (DiagnosticMetrics) obj);
                return updateSynthesizerActive$lambda$2;
            }
        });
        log$default(this, "audio", "synth_state", active ? "Synthesizer started" : "Synthesizer stopped", null, null, 24, null);
    }

    public final void updateState(final VocalAcousticsState state) {
        DiagnosticSeverity diagnosticSeverity;
        String str;
        String f;
        Intrinsics.checkNotNullParameter(state, "state");
        List<Float> formantStdHz = state.getAcoustic().getFormantStdHz();
        if (!(!formantStdHz.isEmpty())) {
            formantStdHz = null;
        }
        final float averageOfFloat = formantStdHz != null ? (float) CollectionsKt.averageOfFloat(formantStdHz) : 0.0f;
        DiagnosticMetrics updateAndGet = metrics.updateAndGet(new UnaryOperator() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda0
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.util.function.Function
            public final Object apply(Object obj) {
                DiagnosticMetrics updateState$lambda$3;
                updateState$lambda$3 = DiagnosticsRuntime.updateState$lambda$3(VocalAcousticsState.this, averageOfFloat, (DiagnosticMetrics) obj);
                return updateState$lambda$3;
            }
        });
        long currentTimeMillis = System.currentTimeMillis();
        if (lastLoggedDrops.getAndSet(updateAndGet.getDroppedFrames()) == updateAndGet.getDroppedFrames() && currentTimeMillis - lastSampleMillis.get() < SAMPLE_INTERVAL_MILLIS) {
            return;
        }
        lastSampleMillis.set(currentTimeMillis);
        DiagnosticAssistant diagnosticAssistant = assistant;
        Intrinsics.checkNotNull(updateAndGet);
        DiagnosticAssessment assess = diagnosticAssistant.assess(updateAndGet, currentTimeMillis);
        DiagnosticFinding diagnosticFinding = (DiagnosticFinding) CollectionsKt.firstOrNull((List) assess.getFindings());
        if (diagnosticFinding == null || (diagnosticSeverity = diagnosticFinding.getSeverity()) == null) {
            diagnosticSeverity = DiagnosticSeverity.INFO;
        }
        DiagnosticSeverity diagnosticSeverity2 = diagnosticSeverity;
        String summary = assess.getSummary();
        Pair[] pairArr = new Pair[13];
        pairArr[0] = TuplesKt.to("source", updateAndGet.getSource());
        pairArr[1] = TuplesKt.to("processing_ms", String.valueOf(updateAndGet.getProcessingMs()));
        pairArr[2] = TuplesKt.to("dropped_frames", String.valueOf(updateAndGet.getDroppedFrames()));
        Float f0Hz = updateAndGet.getF0Hz();
        String str2 = "null";
        if (f0Hz == null || (str = f0Hz.toString()) == null) {
            str = "null";
        }
        pairArr[3] = TuplesKt.to("f0_hz", str);
        Float rawF0Hz = updateAndGet.getRawF0Hz();
        if (rawF0Hz != null && (f = rawF0Hz.toString()) != null) {
            str2 = f;
        }
        pairArr[4] = TuplesKt.to("raw_f0_hz", str2);
        pairArr[5] = TuplesKt.to("pitch_decision", updateAndGet.getPitchDecision());
        pairArr[6] = TuplesKt.to("f0_confidence", String.valueOf(updateAndGet.getF0Confidence()));
        pairArr[7] = TuplesKt.to("snr_db", String.valueOf(updateAndGet.getSnrDb()));
        pairArr[8] = TuplesKt.to("noise_state", updateAndGet.getNoiseState());
        pairArr[9] = TuplesKt.to("background_changed", String.valueOf(updateAndGet.getBackgroundChanged()));
        pairArr[10] = TuplesKt.to("tract_confidence", String.valueOf(updateAndGet.getTractConfidence()));
        pairArr[11] = TuplesKt.to("posterior_abstained", String.valueOf(updateAndGet.getPosteriorAbstained()));
        pairArr[12] = TuplesKt.to("renderer_mode", updateAndGet.getRendererMode());
        log("runtime", "derived_state_sample", summary, MapsKt.mapOf(pairArr), diagnosticSeverity2);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final DiagnosticMetrics updateState$lambda$3(VocalAcousticsState vocalAcousticsState, float f, DiagnosticMetrics diagnosticMetrics) {
        DiagnosticMetrics copy;
        copy = diagnosticMetrics.copy((r50 & 1) != 0 ? diagnosticMetrics.sequence : vocalAcousticsState.getSequence(), (r50 & 2) != 0 ? diagnosticMetrics.source : vocalAcousticsState.getSource(), (r50 & 4) != 0 ? diagnosticMetrics.processingMs : vocalAcousticsState.getProcessingMs(), (r50 & 8) != 0 ? diagnosticMetrics.frameBudgetMs : vocalAcousticsState.getAnalysisHopMs(), (r50 & 16) != 0 ? diagnosticMetrics.droppedFrames : vocalAcousticsState.getDroppedFrames(), (r50 & 32) != 0 ? diagnosticMetrics.voiced : vocalAcousticsState.getAcoustic().getVoiced(), (r50 & 64) != 0 ? diagnosticMetrics.f0Hz : vocalAcousticsState.getAcoustic().getF0Hz(), (r50 & Uuid.SIZE_BITS) != 0 ? diagnosticMetrics.f0Confidence : vocalAcousticsState.getAcoustic().getF0Confidence(), (r50 & 256) != 0 ? diagnosticMetrics.meanFormantStdHz : f, (r50 & ConstantsKt.MINIMUM_BLOCK_SIZE) != 0 ? diagnosticMetrics.tractConfidence : vocalAcousticsState.getTract().getConfidence(), (r50 & 1024) != 0 ? diagnosticMetrics.relativeAreaStd : vocalAcousticsState.getTract().getRelativeAreaStd(), (r50 & 2048) != 0 ? diagnosticMetrics.rendererMode : null, (r50 & ConstantsKt.DEFAULT_BLOCK_SIZE) != 0 ? diagnosticMetrics.microphoneGranted : false, (r50 & ConstantsKt.DEFAULT_BUFFER_SIZE) != 0 ? diagnosticMetrics.synthesizerActive : false, (r50 & 16384) != 0 ? diagnosticMetrics.rawF0Hz : vocalAcousticsState.getAcoustic().getRawF0Hz(), (r50 & 32768) != 0 ? diagnosticMetrics.pitchDecision : vocalAcousticsState.getAcoustic().getPitchDecision(), (r50 & 65536) != 0 ? diagnosticMetrics.pitchRejected : vocalAcousticsState.getAcoustic().getPitchRejected(), (r50 & 131072) != 0 ? diagnosticMetrics.harmonicity : vocalAcousticsState.getAcoustic().getHarmonicity(), (r50 & 262144) != 0 ? diagnosticMetrics.snrDb : vocalAcousticsState.getAcoustic().getSnrDb(), (r50 & 524288) != 0 ? diagnosticMetrics.noiseFloorDb : vocalAcousticsState.getAcoustic().getNoiseFloorDb(), (r50 & 1048576) != 0 ? diagnosticMetrics.noiseState : vocalAcousticsState.getAcoustic().getNoiseState(), (r50 & 2097152) != 0 ? diagnosticMetrics.noiseConfidence : vocalAcousticsState.getAcoustic().getNoiseConfidence(), (r50 & 4194304) != 0 ? diagnosticMetrics.backgroundChanged : vocalAcousticsState.getAcoustic().getBackgroundChanged(), (r50 & 8388608) != 0 ? diagnosticMetrics.noiseBandsDb : ArraysKt.toList(vocalAcousticsState.getAcoustic().getNoiseBandsDb()), (r50 & 16777216) != 0 ? diagnosticMetrics.posteriorAbstained : vocalAcousticsState.getTract().getAbstained(), (r50 & 33554432) != 0 ? diagnosticMetrics.abstentionReason : vocalAcousticsState.getTract().getAbstentionReason(), (r50 & 67108864) != 0 ? diagnosticMetrics.formantCandidatesHz : vocalAcousticsState.getAcoustic().getFormantCandidatesHz(), (r50 & 134217728) != 0 ? diagnosticMetrics.articulators : vocalAcousticsState.getArticulators(), (r50 & 268435456) != 0 ? diagnosticMetrics.analysisWindowMs : vocalAcousticsState.getAnalysisWindowMs(), (r50 & 536870912) != 0 ? diagnosticMetrics.analysisHopMs : vocalAcousticsState.getAnalysisHopMs());
        return copy;
    }

    public static /* synthetic */ void log$default(DiagnosticsRuntime diagnosticsRuntime, String str, String str2, String str3, Map map, DiagnosticSeverity diagnosticSeverity, int i, Object obj) {
        if ((i & 8) != 0) {
            map = MapsKt.emptyMap();
        }
        Map map2 = map;
        if ((i & 16) != 0) {
            diagnosticSeverity = DiagnosticSeverity.INFO;
        }
        diagnosticsRuntime.log(str, str2, str3, map2, diagnosticSeverity);
    }

    public final void log(String category, String code, String message, Map<String, String> evidence, DiagnosticSeverity severity) {
        Intrinsics.checkNotNullParameter(category, "category");
        Intrinsics.checkNotNullParameter(code, "code");
        Intrinsics.checkNotNullParameter(message, "message");
        Intrinsics.checkNotNullParameter(evidence, "evidence");
        Intrinsics.checkNotNullParameter(severity, "severity");
        DiagnosticStore diagnosticStore = store;
        if (diagnosticStore == null) {
            return;
        }
        try {
            Result.Companion companion = Result.INSTANCE;
            DiagnosticsRuntime diagnosticsRuntime = this;
            long currentTimeMillis = System.currentTimeMillis();
            String sessionId = diagnosticStore.getSessionId();
            String take = StringsKt.take(category, 40);
            String take2 = StringsKt.take(code, 80);
            String take3 = StringsKt.take(message, 500);
            LinkedHashMap linkedHashMap = new LinkedHashMap(MapsKt.mapCapacity(evidence.size()));
            for (Object obj : evidence.entrySet()) {
                linkedHashMap.put(((Map.Entry) obj).getKey(), StringsKt.take((String) ((Map.Entry) obj).getValue(), 500));
            }
            Result.m4constructorimpl(Boolean.valueOf(diagnosticStore.append(new DiagnosticEvent(currentTimeMillis, sessionId, take, take2, severity, take3, linkedHashMap))));
        } catch (Throwable th) {
            Result.Companion companion2 = Result.INSTANCE;
            Result.m4constructorimpl(ResultKt.createFailure(th));
        }
    }

    public final RefinementRecord recordRefinement(RefinementDraft draft) {
        Intrinsics.checkNotNullParameter(draft, "draft");
        DiagnosticStore diagnosticStore = store;
        if (diagnosticStore == null) {
            throw new IllegalArgumentException("Diagnostics are not initialized".toString());
        }
        String uuid = UUID.randomUUID().toString();
        Intrinsics.checkNotNullExpressionValue(uuid, "toString(...)");
        long currentTimeMillis = System.currentTimeMillis();
        String sessionId = diagnosticStore.getSessionId();
        DiagnosticMetrics diagnosticMetrics = metrics.get();
        Intrinsics.checkNotNullExpressionValue(diagnosticMetrics, "get(...)");
        RefinementRecord refinementRecord = new RefinementRecord(uuid, currentTimeMillis, sessionId, diagnosticMetrics, draft.validated(), null, 32, null);
        diagnosticStore.appendRefinement(refinementRecord);
        log$default(this, "refinement", "correction_proposed", "A user correction was saved without mutating the model", MapsKt.mapOf(TuplesKt.to("approved_for_future_training", String.valueOf(refinementRecord.getCorrection().getApprovedForFutureTraining()))), null, 16, null);
        return refinementRecord;
    }

    public final void recordCalibration(SingerCalibrationReport report) {
        Intrinsics.checkNotNullParameter(report, "report");
        calibrationReport = report;
        log$default(this, "calibration", "guided_calibration_completed", "Derived-only seven-step singer calibration completed", MapsKt.mapOf(TuplesKt.to("steps", String.valueOf(report.getSteps().size())), TuplesKt.to("raw_audio_stored", String.valueOf(report.getRawAudioStored())), TuplesKt.to("automatic_model_mutation", String.valueOf(report.getAutomaticModelMutation()))), null, 16, null);
    }

    public final List<SelfTestResult> runSelfTests(final Context context) {
        DiagnosticSeverity diagnosticSeverity;
        Intrinsics.checkNotNullParameter(context, "context");
        ArrayList arrayList = new ArrayList();
        runSelfTests$test(arrayList, "reduced_model", new Function0() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda1
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                String runSelfTests$lambda$8;
                runSelfTests$lambda$8 = DiagnosticsRuntime.runSelfTests$lambda$8(context);
                return runSelfTests$lambda$8;
            }
        });
        runSelfTests$test(arrayList, "lumen_asset", new Function0() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda2
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                String runSelfTests$lambda$10;
                runSelfTests$lambda$10 = DiagnosticsRuntime.runSelfTests$lambda$10(context);
                return runSelfTests$lambda$10;
            }
        });
        runSelfTests$test(arrayList, "shared_model_contract", new Function0() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda3
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                String runSelfTests$lambda$11;
                runSelfTests$lambda$11 = DiagnosticsRuntime.runSelfTests$lambda$11(context);
                return runSelfTests$lambda$11;
            }
        });
        runSelfTests$test(arrayList, "synthesis_math", new Function0() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda4
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                String runSelfTests$lambda$15;
                runSelfTests$lambda$15 = DiagnosticsRuntime.runSelfTests$lambda$15();
                return runSelfTests$lambda$15;
            }
        });
        runSelfTests$test(arrayList, "audio_record_format", new Function0() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda5
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                String runSelfTests$lambda$17;
                runSelfTests$lambda$17 = DiagnosticsRuntime.runSelfTests$lambda$17();
                return runSelfTests$lambda$17;
            }
        });
        runSelfTests$test(arrayList, "private_storage", new Function0() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda6
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                String runSelfTests$lambda$19;
                runSelfTests$lambda$19 = DiagnosticsRuntime.runSelfTests$lambda$19();
                return runSelfTests$lambda$19;
            }
        });
        runSelfTests$test(arrayList, "pitch_continuity", new Function0() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda7
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                String runSelfTests$lambda$20;
                runSelfTests$lambda$20 = DiagnosticsRuntime.runSelfTests$lambda$20();
                return runSelfTests$lambda$20;
            }
        });
        runSelfTests$test(arrayList, "stationary_noise", new Function0() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda8
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                String runSelfTests$lambda$24;
                runSelfTests$lambda$24 = DiagnosticsRuntime.runSelfTests$lambda$24();
                return runSelfTests$lambda$24;
            }
        });
        int i = 0;
        boolean z = context.checkSelfPermission("android.permission.RECORD_AUDIO") == 0;
        arrayList.add(new SelfTestResult("microphone_permission", z, z ? "Microphone permission granted" : "Permission not granted; offline tools remain usable"));
        selfTests = arrayList;
        ArrayList arrayList2 = arrayList;
        boolean z2 = arrayList2 instanceof Collection;
        if (!z2 || !arrayList2.isEmpty()) {
            Iterator it = arrayList2.iterator();
            while (it.hasNext()) {
                if (((SelfTestResult) it.next()).getPassed() && (i = i + 1) < 0) {
                    CollectionsKt.throwCountOverflow();
                }
            }
        }
        String str = "Self-test completed: " + i + "/" + arrayList.size() + " passed";
        ArrayList arrayList3 = new ArrayList();
        for (Object obj : arrayList2) {
            if (!((SelfTestResult) obj).getPassed()) {
                arrayList3.add(obj);
            }
        }
        Map<String, String> mapOf = MapsKt.mapOf(TuplesKt.to("failed", CollectionsKt.joinToString$default(arrayList3, null, null, null, 0, null, new Function1() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda9
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function1
            public final Object invoke(Object obj2) {
                CharSequence runSelfTests$lambda$27;
                runSelfTests$lambda$27 = DiagnosticsRuntime.runSelfTests$lambda$27((SelfTestResult) obj2);
                return runSelfTests$lambda$27;
            }
        }, 31, null)));
        if (!z2 || !arrayList2.isEmpty()) {
            Iterator it2 = arrayList2.iterator();
            while (it2.hasNext()) {
                if (!((SelfTestResult) it2.next()).getPassed()) {
                    diagnosticSeverity = DiagnosticSeverity.WARNING;
                    break;
                }
            }
        }
        diagnosticSeverity = DiagnosticSeverity.INFO;
        log("self_test", "self_test_completed", str, mapOf, diagnosticSeverity);
        return arrayList;
    }

    private static final void runSelfTests$test(List<SelfTestResult> list, String str, Function0<String> function0) {
        try {
            list.add(new SelfTestResult(str, true, function0.invoke()));
        } catch (Throwable th) {
            List<SelfTestResult> list2 = list;
            String message = th.getMessage();
            if (message == null) {
                message = th.getClass().getSimpleName();
            }
            Intrinsics.checkNotNull(message);
            list2.add(new SelfTestResult(str, false, message));
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final String runSelfTests$lambda$8(Context context) {
        ReducedModelAsset load = ReducedModelAsset.INSTANCE.load(context);
        if (load.getSchemaVersion() != 2 || load.getHopSize() != 512) {
            throw new IllegalStateException("Expected schema 2 with a 512-sample hop".toString());
        }
        if (load.getScientificReleaseReady() || load.getIndependentExpertAcceptances() != 0) {
            throw new IllegalStateException("Check failed.".toString());
        }
        return load.getModelId() + "; " + load.getSectionPosition().length + " sections; " + load.getModeLabels().size() + " MRI modes";
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final String runSelfTests$lambda$10(Context context) {
        TractLumenAsset load = TractLumenAsset.INSTANCE.load(context);
        return load.getModelId() + "; provenance " + load.getProvenanceKind() + "; " + load.getVertexCount() + " vertices; " + (load.getTriangleIndices().length / 3) + " triangles; sources " + CollectionsKt.joinToString$default(load.getSourceSha256(), null, null, null, 0, null, new Function1() { // from class: org.vocaltract.pixel.DiagnosticsRuntime$$ExternalSyntheticLambda11
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function1
            public final Object invoke(Object obj) {
                CharSequence runSelfTests$lambda$10$lambda$9;
                runSelfTests$lambda$10$lambda$9 = DiagnosticsRuntime.runSelfTests$lambda$10$lambda$9((String) obj);
                return runSelfTests$lambda$10$lambda$9;
            }
        }, 31, null);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final CharSequence runSelfTests$lambda$10$lambda$9(String it) {
        Intrinsics.checkNotNullParameter(it, "it");
        return StringsKt.take(it, 12);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final String runSelfTests$lambda$11(Context context) {
        SharedTractModel sharedTractModel = new SharedTractModel(ReducedModelAsset.INSTANCE.load(context), TractLumenAsset.INSTANCE.load(context));
        SharedLabState sharedLabState = new SharedLabState(sharedTractModel);
        JSONObject snapshot = SharedLabJson.INSTANCE.snapshot(sharedLabState);
        if (!Intrinsics.areEqual(snapshot.getString("modelId"), sharedTractModel.getId()) || snapshot.getBoolean("scientificReleaseReady")) {
            throw new IllegalStateException("Check failed.".toString());
        }
        if (snapshot.getJSONArray("vertices").length() != 2310 || snapshot.getJSONArray("faces").length() != 4608) {
            throw new IllegalStateException("Check failed.".toString());
        }
        if (snapshot.getJSONArray("amplitudes").length() != 16 || snapshot.getJSONArray("responseDb").length() != 497) {
            throw new IllegalStateException("Check failed.".toString());
        }
        if (!SharedLabJson.INSTANCE.restore(sharedLabState, SharedLabJson.INSTANCE.save(sharedLabState))) {
            throw new IllegalStateException("Check failed.".toString());
        }
        if (!Intrinsics.areEqual(LabCommand.INSTANCE.parse("{\"type\":\"stop\",\"requestId\":1}").getType(), "stop")) {
            throw new IllegalStateException("Check failed.".toString());
        }
        return sharedTractModel.getId() + "; native JSON round-trip; shared geometry/response/source contract";
    }

    /* JADX INFO: Access modifiers changed from: private */
    /* JADX WARN: Code restructure failed: missing block: B:10:0x0052, code lost:
    
        throw new java.lang.IllegalStateException("Synthesizer fixture was silent or non-finite".toString());
     */
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    public static final String runSelfTests$lambda$15() {
        float[] summedCycle$default = AdditiveSynthesisMath.summedCycle$default(AdditiveSynthesisMath.INSTANCE, AdditiveSynthState.Companion.create$default(AdditiveSynthState.INSTANCE, 0.0f, 0.0f, null, true, 7, null), SummedWaveformView.DISPLAY_SAMPLE_RATE_HZ, ConstantsKt.MINIMUM_BLOCK_SIZE, 0.0d, 8, null);
        int length = summedCycle$default.length;
        int i = 0;
        while (true) {
            if (i < length) {
                float f = summedCycle$default[i];
                if (Float.isInfinite(f) || Float.isNaN(f)) {
                    break;
                }
                i++;
            } else {
                for (float f2 : summedCycle$default) {
                    if (Math.abs(f2) > 1.0E-5f) {
                        return "512 finite synthesis samples";
                    }
                }
            }
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final String runSelfTests$lambda$17() {
        int minBufferSize = AudioRecord.getMinBufferSize(16000, 16, 2);
        if (minBufferSize <= 0) {
            throw new IllegalStateException(("16 kHz mono PCM16 is unsupported: " + minBufferSize).toString());
        }
        return "minimum buffer " + minBufferSize + " bytes; capture not started";
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final String runSelfTests$lambda$19() {
        Context context = applicationContext;
        if (context == null) {
            throw new IllegalArgumentException("Required value was null.".toString());
        }
        File filesDir = context.getFilesDir();
        if (!filesDir.canWrite()) {
            throw new IllegalStateException("Internal app storage is not writable".toString());
        }
        return (filesDir.getUsableSpace() / 1048576) + " MiB available";
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final String runSelfTests$lambda$20() {
        PitchContinuityGate pitchContinuityGate = new PitchContinuityGate();
        pitchContinuityGate.update(Float.valueOf(106.7363f), 0.858f, 0.82f);
        PitchDecision update = pitchContinuityGate.update(Float.valueOf(603.0734f), 0.47234312f, 0.44f);
        if (!Intrinsics.areEqual(update.getDecision(), "harmonic_alias_corrected")) {
            throw new IllegalStateException("Check failed.".toString());
        }
        Float acceptedF0Hz = update.getAcceptedF0Hz();
        if (acceptedF0Hz == null) {
            throw new IllegalArgumentException("Required value was null.".toString());
        }
        float floatValue = acceptedF0Hz.floatValue();
        if (105.0f > floatValue || floatValue > 116.0f) {
            throw new IllegalStateException("Check failed.".toString());
        }
        return "Physical diagnostic regression corrected to " + update.getAcceptedF0Hz() + " Hz";
    }

    /* JADX DEBUG: Class process forced to load method for inline: org.vocaltract.pixel.StationaryNoiseTracker.update$default(org.vocaltract.pixel.StationaryNoiseTracker, float[], boolean, boolean, int, java.lang.Object):org.vocaltract.pixel.NoiseEstimate */
    /* JADX INFO: Access modifiers changed from: private */
    public static final String runSelfTests$lambda$24() {
        StationaryNoiseTracker stationaryNoiseTracker = new StationaryNoiseTracker(32, 0, 2, null);
        float[] fArr = new float[32];
        for (int i = 0; i < 32; i++) {
            fArr[i] = 0.01f;
        }
        fArr[3] = 2.0f;
        for (int i2 = 0; i2 < 40; i2++) {
            stationaryNoiseTracker.update(fArr, false, true);
        }
        float[] copyOf = Arrays.copyOf(fArr, 32);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        copyOf[8] = copyOf[8] + 30.0f;
        NoiseEstimate update$default = StationaryNoiseTracker.update$default(stationaryNoiseTracker, copyOf, true, false, 4, null);
        if (update$default.getConfidence() <= 0.8f || update$default.getCleanPower()[8] <= update$default.getCleanPower()[3]) {
            throw new IllegalStateException("Check failed.".toString());
        }
        return "Fan fixture learned; singer residual retained; SNR " + update$default.getSnrDb() + " dB";
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final CharSequence runSelfTests$lambda$27(SelfTestResult it) {
        Intrinsics.checkNotNullParameter(it, "it");
        return it.getCode();
    }

    public final DiagnosticExport exportBundle() {
        DiagnosticStore diagnosticStore = store;
        if (diagnosticStore == null) {
            throw new IllegalArgumentException("Diagnostics are not initialized".toString());
        }
        Context context = applicationContext;
        if (context == null) {
            throw new IllegalArgumentException("Required value was null.".toString());
        }
        if (selfTests.isEmpty()) {
            runSelfTests(context);
        }
        DiagnosticMetrics diagnosticMetrics = metrics.get();
        Intrinsics.checkNotNullExpressionValue(diagnosticMetrics, "get(...)");
        DiagnosticExport exportBundle = diagnosticStore.exportBundle(diagnosticMetrics, currentAssessment(), selfTests, calibrationReport, sharedModelSnapshot);
        log$default(this, "export", "bundle_exported", "Privacy-safe diagnostic bundle exported", MapsKt.mapOf(TuplesKt.to("display_name", exportBundle.getDisplayName()), TuplesKt.to("bytes", String.valueOf(exportBundle.getBytes()))), null, 16, null);
        return exportBundle;
    }

    public final void clearLogs() {
        DiagnosticStore diagnosticStore = store;
        if (diagnosticStore != null) {
            diagnosticStore.clear();
        }
        selfTests = CollectionsKt.emptyList();
        calibrationReport = null;
        log$default(this, "privacy", "logs_cleared", "Local diagnostic history was cleared by the user", null, null, 24, null);
    }

    public final boolean debugOverlayEnabled(Context context) {
        Intrinsics.checkNotNullParameter(context, "context");
        return preferences(context).getBoolean(OVERLAY_KEY, false);
    }

    public final void setDebugOverlayEnabled(Context context, boolean enabled) {
        Intrinsics.checkNotNullParameter(context, "context");
        preferences(context).edit().putBoolean(OVERLAY_KEY, enabled).apply();
        log$default(this, "ui", "debug_overlay", "Live debug overlay ".concat(enabled ? "enabled" : "disabled"), null, null, 24, null);
    }

    private final SharedPreferences preferences(Context context) {
        return context.getSharedPreferences("diagnostics_preferences", 0);
    }
}

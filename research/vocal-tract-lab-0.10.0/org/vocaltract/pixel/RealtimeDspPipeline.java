package org.vocaltract.pixel;

import android.os.Process;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicLong;
import kotlin.Metadata;
import kotlin.Unit;
import kotlin.concurrent.ThreadsKt;
import kotlin.jvm.functions.Function0;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.functions.Function2;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: RealtimeDspPipeline.kt */
@Metadata(d1 = {"\u0000r\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\b\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\u0010\t\n\u0002\u0010\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\u0010\u000e\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0005\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010\u0014\n\u0002\b\u0006\u0018\u00002\u00020\u0001B]\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\b\b\u0002\u0010\u0004\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0006\u001a\u00020\u0005\u0012\n\b\u0002\u0010\u0007\u001a\u0004\u0018\u00010\b\u0012\u0018\u0010\t\u001a\u0014\u0012\u0004\u0012\u00020\u000b\u0012\u0004\u0012\u00020\f\u0012\u0004\u0012\u00020\r0\n\u0012\u0012\u0010\u000e\u001a\u000e\u0012\u0004\u0012\u00020\u0010\u0012\u0004\u0012\u00020\r0\u000f¢\u0006\u0004\b\u0011\u0010\u0012J\u0016\u0010#\u001a\u00020$2\u0006\u0010%\u001a\u00020\f2\u0006\u0010&\u001a\u00020'J\u000e\u0010(\u001a\u00020\r2\u0006\u0010)\u001a\u00020\u0015J\u0010\u0010*\u001a\u00020\r2\b\b\u0002\u0010+\u001a\u00020\u0005J\u0010\u0010,\u001a\u00020\r2\b\b\u0002\u0010-\u001a\u00020$R\u000e\u0010\u0002\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0004\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0006\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u0010\u0010\u0007\u001a\u0004\u0018\u00010\bX\u0082\u0004¢\u0006\u0002\n\u0000R \u0010\t\u001a\u0014\u0012\u0004\u0012\u00020\u000b\u0012\u0004\u0012\u00020\f\u0012\u0004\u0012\u00020\r0\nX\u0082\u0004¢\u0006\u0002\n\u0000R\u001a\u0010\u000e\u001a\u000e\u0012\u0004\u0012\u00020\u0010\u0012\u0004\u0012\u00020\r0\u000fX\u0082\u0004¢\u0006\u0002\n\u0000R\u0014\u0010\u0013\u001a\b\u0012\u0004\u0012\u00020\u00150\u0014X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0016\u001a\u00020\u0017X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0018\u001a\u00020\u0019X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u001a\u001a\u00020\u0019X\u0082\u0004¢\u0006\u0002\n\u0000R\u0010\u0010\u001b\u001a\u0004\u0018\u00010\u001cX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u001d\u001a\u00020\u001eX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u001f\u001a\u00020\u0019X\u0082\u0004¢\u0006\u0002\n\u0000R\u0011\u0010 \u001a\u00020\f8F¢\u0006\u0006\u001a\u0004\b!\u0010\""}, d2 = {"Lorg/vocaltract/pixel/RealtimeDspPipeline;", "", "asset", "Lorg/vocaltract/pixel/ReducedModelAsset;", "queueCapacity", "", "maxHarmonics", "sharedModel", "Lorg/vocaltract/pixel/SharedTractModel;", "onState", "Lkotlin/Function2;", "Lorg/vocaltract/pixel/VocalAcousticsState;", "", "", "onError", "Lkotlin/Function1;", "", "<init>", "(Lorg/vocaltract/pixel/ReducedModelAsset;IILorg/vocaltract/pixel/SharedTractModel;Lkotlin/jvm/functions/Function2;Lkotlin/jvm/functions/Function1;)V", "queue", "Ljava/util/concurrent/ArrayBlockingQueue;", "Lorg/vocaltract/pixel/PcmFrame;", "running", "Ljava/util/concurrent/atomic/AtomicBoolean;", "dropped", "Ljava/util/concurrent/atomic/AtomicLong;", "sequence", "worker", "Ljava/lang/Thread;", "analyzer", "Lorg/vocaltract/pixel/FrameAnalyzer;", "generation", "droppedFrames", "getDroppedFrames", "()J", "start", "", "sessionToken", "seed", "", "offer", "frame", "requestNoiseLearning", "frames", "stop", "drain"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class RealtimeDspPipeline {
    private volatile FrameAnalyzer analyzer;
    private final ReducedModelAsset asset;
    private final AtomicLong dropped;
    private final AtomicLong generation;
    private final int maxHarmonics;
    private final Function1<String, Unit> onError;
    private final Function2<VocalAcousticsState, Long, Unit> onState;
    private final ArrayBlockingQueue<PcmFrame> queue;
    private final int queueCapacity;
    private final AtomicBoolean running;
    private final AtomicLong sequence;
    private final SharedTractModel sharedModel;
    private volatile Thread worker;

    /* JADX DEBUG: Multi-variable search result rejected for r12v0, resolved type: kotlin.jvm.functions.Function2<? super org.vocaltract.pixel.VocalAcousticsState, ? super java.lang.Long, kotlin.Unit> */
    /* JADX DEBUG: Multi-variable search result rejected for r13v0, resolved type: kotlin.jvm.functions.Function1<? super java.lang.String, kotlin.Unit> */
    /* JADX WARN: Multi-variable type inference failed */
    public RealtimeDspPipeline(ReducedModelAsset asset, int i, int i2, SharedTractModel sharedTractModel, Function2<? super VocalAcousticsState, ? super Long, Unit> onState, Function1<? super String, Unit> onError) {
        Intrinsics.checkNotNullParameter(asset, "asset");
        Intrinsics.checkNotNullParameter(onState, "onState");
        Intrinsics.checkNotNullParameter(onError, "onError");
        this.asset = asset;
        this.queueCapacity = i;
        this.maxHarmonics = i2;
        this.sharedModel = sharedTractModel;
        this.onState = onState;
        this.onError = onError;
        this.queue = new ArrayBlockingQueue<>(i);
        this.running = new AtomicBoolean(false);
        this.dropped = new AtomicLong(0L);
        this.sequence = new AtomicLong(0L);
        this.analyzer = new FrameAnalyzer(asset, i2, sharedTractModel, null, 8, null);
        this.generation = new AtomicLong(0L);
    }

    public /* synthetic */ RealtimeDspPipeline(ReducedModelAsset reducedModelAsset, int i, int i2, SharedTractModel sharedTractModel, Function2 function2, Function1 function1, int i3, DefaultConstructorMarker defaultConstructorMarker) {
        this(reducedModelAsset, (i3 & 2) != 0 ? 4 : i, (i3 & 4) != 0 ? 16 : i2, (i3 & 8) != 0 ? null : sharedTractModel, function2, function1);
    }

    public final long getDroppedFrames() {
        return this.dropped.get();
    }

    public final synchronized boolean start(final long sessionToken, float[] seed) {
        Intrinsics.checkNotNullParameter(seed, "seed");
        Thread thread = this.worker;
        if ((thread == null || !thread.isAlive()) && this.running.compareAndSet(false, true)) {
            this.queue.clear();
            this.analyzer = new FrameAnalyzer(this.asset, this.maxHarmonics, this.sharedModel, seed);
            final FrameAnalyzer frameAnalyzer = this.analyzer;
            final long incrementAndGet = this.generation.incrementAndGet();
            this.worker = ThreadsKt.thread$default(false, true, null, "vocal-dsp", 0, new Function0() { // from class: org.vocaltract.pixel.RealtimeDspPipeline$$ExternalSyntheticLambda0
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function0
                public final Object invoke() {
                    Unit start$lambda$0;
                    start$lambda$0 = RealtimeDspPipeline.start$lambda$0(RealtimeDspPipeline.this, incrementAndGet, frameAnalyzer, sessionToken);
                    return start$lambda$0;
                }
            }, 21, null);
            return true;
        }
        return false;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit start$lambda$0(RealtimeDspPipeline realtimeDspPipeline, long j, FrameAnalyzer frameAnalyzer, long j2) {
        Process.setThreadPriority(-16);
        while (realtimeDspPipeline.generation.get() == j && (realtimeDspPipeline.running.get() || (!realtimeDspPipeline.queue.isEmpty()))) {
            PcmFrame poll = realtimeDspPipeline.queue.poll(100L, TimeUnit.MILLISECONDS);
            if (poll != null) {
                try {
                    VocalAcousticsState analyze = frameAnalyzer.analyze(poll, realtimeDspPipeline.sequence.getAndIncrement(), realtimeDspPipeline.dropped.get());
                    if (realtimeDspPipeline.generation.get() == j) {
                        realtimeDspPipeline.onState.invoke(analyze, Long.valueOf(j2));
                    }
                } catch (RuntimeException e) {
                    realtimeDspPipeline.onError.invoke("DSP frame failed: " + e.getMessage());
                }
            }
        }
        return Unit.INSTANCE;
    }

    public final void offer(PcmFrame frame) {
        Intrinsics.checkNotNullParameter(frame, "frame");
        if (this.running.get() && !this.queue.offer(frame)) {
            this.queue.poll();
            this.dropped.incrementAndGet();
            this.queue.offer(frame);
        }
    }

    public static /* synthetic */ void requestNoiseLearning$default(RealtimeDspPipeline realtimeDspPipeline, int i, int i2, Object obj) {
        if ((i2 & 1) != 0) {
            i = 48;
        }
        realtimeDspPipeline.requestNoiseLearning(i);
    }

    public final void requestNoiseLearning(int frames) {
        this.analyzer.requestNoiseLearning(frames);
    }

    public static /* synthetic */ void stop$default(RealtimeDspPipeline realtimeDspPipeline, boolean z, int i, Object obj) {
        if ((i & 1) != 0) {
            z = true;
        }
        realtimeDspPipeline.stop(z);
    }

    public final synchronized void stop(boolean drain) {
        if (drain) {
            long nanoTime = System.nanoTime() + 1000000000;
            while ((!this.queue.isEmpty()) && System.nanoTime() < nanoTime) {
                Thread.yield();
            }
        } else {
            this.queue.clear();
        }
        this.running.set(false);
        this.generation.incrementAndGet();
        Thread thread = this.worker;
        if (thread != null) {
            thread.join(1500L);
        }
        Thread thread2 = this.worker;
        if (thread2 == null || !thread2.isAlive()) {
            this.worker = null;
        }
    }
}

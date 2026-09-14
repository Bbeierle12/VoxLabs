package org.vocaltract.pixel;

import java.util.Iterator;
import java.util.concurrent.atomic.AtomicBoolean;
import kotlin.Metadata;
import kotlin.Unit;
import kotlin.concurrent.ThreadsKt;
import kotlin.jvm.functions.Function0;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.Intrinsics;
import kotlin.sequences.Sequence;

/* compiled from: ReplaySources.kt */
@Metadata(d1 = {"\u0000<\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\b\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\u0010\u0002\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u000b\n\u0000\u0018\u00002\u00020\u0001B+\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0012\u0010\u0005\u001a\u000e\u0012\u0004\u0012\u00020\u0007\u0012\u0004\u0012\u00020\b0\u0006¢\u0006\u0004\b\t\u0010\nJ\u001e\u0010\u000f\u001a\u00020\b2\f\u0010\u0010\u001a\b\u0012\u0004\u0012\u00020\u00070\u00112\b\b\u0002\u0010\u0012\u001a\u00020\u0013J\u0006\u0010\u0014\u001a\u00020\bR\u000e\u0010\u0002\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0004\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u001a\u0010\u0005\u001a\u000e\u0012\u0004\u0012\u00020\u0007\u0012\u0004\u0012\u00020\b0\u0006X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u000b\u001a\u00020\fX\u0082\u0004¢\u0006\u0002\n\u0000R\u0010\u0010\r\u001a\u0004\u0018\u00010\u000eX\u0082\u000e¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/ReplayController;", "", "sampleRateHz", "", "frameSize", "sink", "Lkotlin/Function1;", "Lorg/vocaltract/pixel/PcmFrame;", "", "<init>", "(IILkotlin/jvm/functions/Function1;)V", "running", "Ljava/util/concurrent/atomic/AtomicBoolean;", "worker", "Ljava/lang/Thread;", "play", "frames", "Lkotlin/sequences/Sequence;", "realTime", "", "stop"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class ReplayController {
    private final int frameSize;
    private final AtomicBoolean running;
    private final int sampleRateHz;
    private final Function1<PcmFrame, Unit> sink;
    private volatile Thread worker;

    /* JADX DEBUG: Multi-variable search result rejected for r4v0, resolved type: kotlin.jvm.functions.Function1<? super org.vocaltract.pixel.PcmFrame, kotlin.Unit> */
    /* JADX WARN: Multi-variable type inference failed */
    public ReplayController(int i, int i2, Function1<? super PcmFrame, Unit> sink) {
        Intrinsics.checkNotNullParameter(sink, "sink");
        this.sampleRateHz = i;
        this.frameSize = i2;
        this.sink = sink;
        this.running = new AtomicBoolean(false);
    }

    public static /* synthetic */ void play$default(ReplayController replayController, Sequence sequence, boolean z, int i, Object obj) {
        if ((i & 2) != 0) {
            z = true;
        }
        replayController.play(sequence, z);
    }

    public final void play(final Sequence<PcmFrame> frames, final boolean realTime) {
        Intrinsics.checkNotNullParameter(frames, "frames");
        stop();
        this.running.set(true);
        this.worker = ThreadsKt.thread$default(false, true, null, "offline-replay", 0, new Function0() { // from class: org.vocaltract.pixel.ReplayController$$ExternalSyntheticLambda0
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                Unit play$lambda$0;
                play$lambda$0 = ReplayController.play$lambda$0(ReplayController.this, frames, realTime);
                return play$lambda$0;
            }
        }, 21, null);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit play$lambda$0(ReplayController replayController, Sequence sequence, boolean z) {
        long j = (replayController.frameSize * 1000000000) / replayController.sampleRateHz;
        long nanoTime = System.nanoTime();
        Iterator it = sequence.iterator();
        while (it.hasNext()) {
            PcmFrame pcmFrame = (PcmFrame) it.next();
            if (!replayController.running.get()) {
                break;
            }
            replayController.sink.invoke(PcmFrame.copy$default(pcmFrame, System.nanoTime(), null, null, 6, null));
            if (z) {
                nanoTime += j;
                long nanoTime2 = nanoTime - System.nanoTime();
                if (nanoTime2 > 0) {
                    try {
                        Thread.sleep(nanoTime2 / 1000000, (int) (nanoTime2 % 1000000));
                    } catch (InterruptedException unused) {
                    }
                } else {
                    continue;
                }
            }
        }
        replayController.running.set(false);
        return Unit.INSTANCE;
    }

    public final void stop() {
        if (this.running.getAndSet(false)) {
            Thread thread = this.worker;
            if (thread != null) {
                thread.interrupt();
            }
            Thread thread2 = this.worker;
            if (thread2 != null) {
                thread2.join(500L);
            }
            this.worker = null;
        }
    }
}

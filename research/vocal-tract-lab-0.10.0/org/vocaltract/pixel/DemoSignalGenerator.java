package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.sequences.Sequence;
import kotlin.sequences.SequencesKt;

/* compiled from: ReplaySources.kt */
@Metadata(d1 = {"\u0000,\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\b\n\u0002\b\u0002\n\u0002\u0010\u0007\n\u0002\b\u0002\n\u0002\u0010\u0006\n\u0000\bÆ\u0002\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J0\u0010\u0004\u001a\b\u0012\u0004\u0012\u00020\u00060\u00052\u0006\u0010\u0007\u001a\u00020\b2\u0006\u0010\t\u001a\u00020\b2\b\b\u0002\u0010\n\u001a\u00020\u000b2\b\b\u0002\u0010\f\u001a\u00020\u000bJ\u0010\u0010\r\u001a\u00020\u000e2\u0006\u0010\u000f\u001a\u00020\u000eH\u0002"}, d2 = {"Lorg/vocaltract/pixel/DemoSignalGenerator;", "", "<init>", "()V", "frames", "Lkotlin/sequences/Sequence;", "Lorg/vocaltract/pixel/PcmFrame;", "sampleRateHz", "", "frameSize", "f0Hz", "", "durationSeconds", "sq", "", "value"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class DemoSignalGenerator {
    public static final DemoSignalGenerator INSTANCE = new DemoSignalGenerator();

    /* JADX INFO: Access modifiers changed from: private */
    public final double sq(double value) {
        return value * value;
    }

    private DemoSignalGenerator() {
    }

    public static /* synthetic */ Sequence frames$default(DemoSignalGenerator demoSignalGenerator, int i, int i2, float f, float f2, int i3, Object obj) {
        if ((i3 & 4) != 0) {
            f = 180.0f;
        }
        if ((i3 & 8) != 0) {
            f2 = 2.0f;
        }
        return demoSignalGenerator.frames(i, i2, f, f2);
    }

    public final Sequence<PcmFrame> frames(int sampleRateHz, int frameSize, float f0Hz, float durationSeconds) {
        return SequencesKt.sequence(new DemoSignalGenerator$frames$1(sampleRateHz, frameSize, f0Hz, durationSeconds, null));
    }
}

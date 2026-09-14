package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;
import kotlin.sequences.Sequence;
import kotlin.sequences.SequencesKt;

/* compiled from: AnatomyBenchActivity.kt */
@Metadata(d1 = {"\u0000$\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\b\n\u0002\b\u0002\bÆ\u0002\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J,\u0010\u0004\u001a\b\u0012\u0004\u0012\u00020\u00060\u00052\u0006\u0010\u0007\u001a\u00020\b2\u0006\u0010\t\u001a\u00020\n2\u0006\u0010\u000b\u001a\u00020\n2\u0006\u0010\f\u001a\u00020\n"}, d2 = {"Lorg/vocaltract/pixel/SharedReplay;", "", "<init>", "()V", "frames", "Lkotlin/sequences/Sequence;", "Lorg/vocaltract/pixel/PcmFrame;", "sound", "Lorg/vocaltract/pixel/AdditiveSynthState;", "rate", "", "size", "hop"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class SharedReplay {
    public static final SharedReplay INSTANCE = new SharedReplay();

    private SharedReplay() {
    }

    public final Sequence<PcmFrame> frames(AdditiveSynthState sound, int rate, int size, int hop) {
        Intrinsics.checkNotNullParameter(sound, "sound");
        return SequencesKt.sequence(new SharedReplay$frames$1(rate, size, sound, hop, null));
    }
}

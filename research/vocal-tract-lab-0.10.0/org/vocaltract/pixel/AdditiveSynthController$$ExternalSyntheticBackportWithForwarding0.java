package org.vocaltract.pixel;

import java.util.concurrent.atomic.AtomicReference;

/* compiled from: D8$$SyntheticClass */
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* synthetic */ class AdditiveSynthController$$ExternalSyntheticBackportWithForwarding0 {
    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
    public static /* synthetic */ boolean m(AtomicReference atomicReference, Object obj, Object obj2) {
        while (!atomicReference.compareAndSet(obj, obj2)) {
            if (atomicReference.get() != obj) {
                return false;
            }
        }
        return true;
    }
}

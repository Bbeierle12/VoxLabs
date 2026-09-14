package org.vocaltract.pixel;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Iterator;
import java.util.List;
import kotlin.Metadata;
import kotlin.collections.CollectionsKt;
import kotlin.jvm.internal.Intrinsics;
import org.vocaltract.pixel.AdditiveSynthState;

/* compiled from: LabAudioContract.kt */
@Metadata(d1 = {"\u0000$\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0006\n\u0002\b\u0002\n\u0002\u0010 \n\u0000\n\u0002\u0010\u000b\bÆ\u0002\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J,\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u00072\u0006\u0010\b\u001a\u00020\u00072\f\u0010\t\u001a\b\u0012\u0004\u0012\u00020\u00070\n2\u0006\u0010\u000b\u001a\u00020\f"}, d2 = {"Lorg/vocaltract/pixel/LabAudioContract;", "", "<init>", "()V", "state", "Lorg/vocaltract/pixel/AdditiveSynthState;", "f0", "", "master", "gains", "", "playing", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class LabAudioContract {
    public static final LabAudioContract INSTANCE = new LabAudioContract();

    private LabAudioContract() {
    }

    public final AdditiveSynthState state(double f0, double master, List<Double> gains, boolean playing) {
        Intrinsics.checkNotNullParameter(gains, "gains");
        if (Double.isInfinite(f0) || Double.isNaN(f0) || 55.0d > f0 || f0 > 800.0d) {
            throw new IllegalArgumentException("F0 outside lab range".toString());
        }
        if (Double.isInfinite(master) || Double.isNaN(master) || 0.0d > master || master > 0.92d) {
            throw new IllegalArgumentException("Invalid output level".toString());
        }
        if (gains.size() == 16) {
            List<Double> list = gains;
            if (!(list instanceof Collection) || !list.isEmpty()) {
                Iterator<T> it = list.iterator();
                while (it.hasNext()) {
                    double doubleValue = ((Number) it.next()).doubleValue();
                    if (!Double.isInfinite(doubleValue) && !Double.isNaN(doubleValue) && 0.0d <= doubleValue && doubleValue <= 1.0d) {
                    }
                }
            }
            AdditiveSynthState.Companion companion = AdditiveSynthState.INSTANCE;
            float f = (float) f0;
            float f2 = (float) master;
            List<Double> list2 = gains;
            ArrayList arrayList = new ArrayList(CollectionsKt.collectionSizeOrDefault(list2, 10));
            Iterator<T> it2 = list2.iterator();
            while (it2.hasNext()) {
                arrayList.add(Float.valueOf((float) ((Number) it2.next()).doubleValue()));
            }
            return companion.create(f, f2, arrayList, playing);
        }
        throw new IllegalArgumentException("Invalid partial gains".toString());
    }
}

package org.vocaltract.pixel;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import kotlin.Metadata;
import kotlin.collections.ArraysKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: AdditiveSynthesis.kt */
@Metadata(d1 = {"\u00006\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0002\n\u0002\u0010\u0014\n\u0000\n\u0002\u0010\u000b\n\u0002\b\b\n\u0002\u0010\b\n\u0002\b\u0005\n\u0002\u0010 \n\u0002\b\b\n\u0002\u0010\u000e\n\u0000\u0018\u0000 !2\u00020\u0001:\u0001!B)\b\u0002\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0006\u0012\u0006\u0010\u0007\u001a\u00020\b¢\u0006\u0004\b\t\u0010\nJ\u000e\u0010\u0014\u001a\u00020\u00032\u0006\u0010\u0015\u001a\u00020\u0011J\f\u0010\u0016\u001a\b\u0012\u0004\u0012\u00020\u00030\u0017J\r\u0010\u0018\u001a\u00020\u0006H\u0000¢\u0006\u0002\b\u0019J\u000e\u0010\u001a\u001a\u00020\u00002\u0006\u0010\u001b\u001a\u00020\u0003J\u000e\u0010\u001c\u001a\u00020\u00002\u0006\u0010\u001b\u001a\u00020\u0003J\u0016\u0010\u001d\u001a\u00020\u00002\u0006\u0010\u0015\u001a\u00020\u00112\u0006\u0010\u001b\u001a\u00020\u0003J\u000e\u0010\u001e\u001a\u00020\u00002\u0006\u0010\u001b\u001a\u00020\bJ\b\u0010\u001f\u001a\u00020 H\u0016R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000b\u0010\fR\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\r\u0010\fR\u000e\u0010\u0005\u001a\u00020\u0006X\u0082\u0004¢\u0006\u0002\n\u0000R\u0011\u0010\u0007\u001a\u00020\b¢\u0006\b\n\u0000\u001a\u0004\b\u000e\u0010\u000fR\u0011\u0010\u0010\u001a\u00020\u00118F¢\u0006\u0006\u001a\u0004\b\u0012\u0010\u0013"}, d2 = {"Lorg/vocaltract/pixel/AdditiveSynthState;", "", "fundamentalHz", "", "masterGain", "gainValues", "", "playing", "", "<init>", "(FF[FZ)V", "getFundamentalHz", "()F", "getMasterGain", "getPlaying", "()Z", "partialCount", "", "getPartialCount", "()I", "partialGain", "index", "partialGains", "", "copyPartialGains", "copyPartialGains$main", "withFundamentalHz", "value", "withMasterGain", "withPartialGain", "withPlaying", "toString", "", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class AdditiveSynthState {

    /* renamed from: Companion, reason: from kotlin metadata */
    public static final Companion INSTANCE = new Companion(null);
    public static final float DEFAULT_FUNDAMENTAL_HZ = 220.0f;
    public static final float MAX_FUNDAMENTAL_HZ = 4000.0f;
    public static final float MAX_MASTER_GAIN = 0.92f;
    public static final float MIN_FUNDAMENTAL_HZ = 20.0f;
    public static final int PARTIAL_COUNT = 16;
    private final float fundamentalHz;
    private final float[] gainValues;
    private final float masterGain;
    private final boolean playing;

    public /* synthetic */ AdditiveSynthState(float f, float f2, float[] fArr, boolean z, DefaultConstructorMarker defaultConstructorMarker) {
        this(f, f2, fArr, z);
    }

    public final int getPartialCount() {
        return 16;
    }

    private AdditiveSynthState(float f, float f2, float[] fArr, boolean z) {
        this.fundamentalHz = f;
        this.masterGain = f2;
        this.gainValues = fArr;
        this.playing = z;
    }

    public final float getFundamentalHz() {
        return this.fundamentalHz;
    }

    public final float getMasterGain() {
        return this.masterGain;
    }

    public final boolean getPlaying() {
        return this.playing;
    }

    public final float partialGain(int index) {
        if (index < 0 || index >= 16) {
            throw new IllegalArgumentException("Partial index must be in 0..15".toString());
        }
        return this.gainValues[index];
    }

    public final List<Float> partialGains() {
        List<Float> unmodifiableList = Collections.unmodifiableList(ArraysKt.toList(this.gainValues));
        Intrinsics.checkNotNullExpressionValue(unmodifiableList, "unmodifiableList(...)");
        return unmodifiableList;
    }

    public final float[] copyPartialGains$main() {
        float[] fArr = this.gainValues;
        float[] copyOf = Arrays.copyOf(fArr, fArr.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        return copyOf;
    }

    public final AdditiveSynthState withFundamentalHz(float value) {
        return INSTANCE.create(value, this.masterGain, ArraysKt.asList(this.gainValues), this.playing);
    }

    public final AdditiveSynthState withMasterGain(float value) {
        return INSTANCE.create(this.fundamentalHz, value, ArraysKt.asList(this.gainValues), this.playing);
    }

    public final AdditiveSynthState withPartialGain(int index, float value) {
        if (index < 0 || index >= 16) {
            throw new IllegalArgumentException("Partial index must be in 0..15".toString());
        }
        float[] fArr = this.gainValues;
        float[] copyOf = Arrays.copyOf(fArr, fArr.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        copyOf[index] = INSTANCE.sanitizeUnit(value);
        return new AdditiveSynthState(this.fundamentalHz, this.masterGain, copyOf, this.playing);
    }

    public final AdditiveSynthState withPlaying(boolean value) {
        float f = this.fundamentalHz;
        float f2 = this.masterGain;
        float[] fArr = this.gainValues;
        float[] copyOf = Arrays.copyOf(fArr, fArr.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        return new AdditiveSynthState(f, f2, copyOf, value);
    }

    public String toString() {
        float f = this.fundamentalHz;
        float f2 = this.masterGain;
        boolean z = this.playing;
        String arrays = Arrays.toString(this.gainValues);
        Intrinsics.checkNotNullExpressionValue(arrays, "toString(...)");
        return "AdditiveSynthState(fundamentalHz=" + f + ", masterGain=" + f2 + ", playing=" + z + ", partials=" + arrays + ")";
    }

    /* compiled from: AdditiveSynthesis.kt */
    @Metadata(d1 = {"\u00000\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0004\n\u0002\u0010 \n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0004\n\u0002\u0010\u000b\n\u0002\b\u0002\b\u0086\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\f\u0010\u000b\u001a\b\u0012\u0004\u0012\u00020\u00070\fJ4\u0010\r\u001a\u00020\u000e2\b\b\u0002\u0010\u000f\u001a\u00020\u00072\b\b\u0002\u0010\u0010\u001a\u00020\u00072\u000e\b\u0002\u0010\u0011\u001a\b\u0012\u0004\u0012\u00020\u00070\f2\b\b\u0002\u0010\u0012\u001a\u00020\u0013J\u0010\u0010\u0014\u001a\u00020\u00072\u0006\u0010\u0015\u001a\u00020\u0007H\u0002R\u000e\u0010\u0004\u001a\u00020\u0005X\u0086T¢\u0006\u0002\n\u0000R\u000e\u0010\u0006\u001a\u00020\u0007X\u0086T¢\u0006\u0002\n\u0000R\u000e\u0010\b\u001a\u00020\u0007X\u0086T¢\u0006\u0002\n\u0000R\u000e\u0010\t\u001a\u00020\u0007X\u0086T¢\u0006\u0002\n\u0000R\u000e\u0010\n\u001a\u00020\u0007X\u0086T¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/AdditiveSynthState$Companion;", "", "<init>", "()V", "PARTIAL_COUNT", "", "DEFAULT_FUNDAMENTAL_HZ", "", "MIN_FUNDAMENTAL_HZ", "MAX_FUNDAMENTAL_HZ", "MAX_MASTER_GAIN", "defaultPartialGains", "", "create", "Lorg/vocaltract/pixel/AdditiveSynthState;", "fundamentalHz", "masterGain", "partialGains", "playing", "", "sanitizeUnit", "value"}, k = 1, mv = {2, 0, 0}, xi = 48)
    public static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }

        public final List<Float> defaultPartialGains() {
            ArrayList arrayList = new ArrayList(16);
            int i = 0;
            while (i < 16) {
                i++;
                arrayList.add(Float.valueOf(1.0f / i));
            }
            return arrayList;
        }

        /* JADX DEBUG: Multi-variable search result rejected for r0v0, resolved type: org.vocaltract.pixel.AdditiveSynthState$Companion */
        /* JADX WARN: Multi-variable type inference failed */
        public static /* synthetic */ AdditiveSynthState create$default(Companion companion, float f, float f2, List list, boolean z, int i, Object obj) {
            if ((i & 1) != 0) {
                f = 220.0f;
            }
            if ((i & 2) != 0) {
                f2 = 0.65f;
            }
            if ((i & 4) != 0) {
                list = companion.defaultPartialGains();
            }
            if ((i & 8) != 0) {
                z = false;
            }
            return companion.create(f, f2, list, z);
        }

        public final AdditiveSynthState create(float fundamentalHz, float masterGain, List<Float> partialGains, boolean playing) {
            Intrinsics.checkNotNullParameter(partialGains, "partialGains");
            if (partialGains.size() != 16) {
                throw new IllegalArgumentException("Exactly 16 partial gains are required".toString());
            }
            float coerceIn = (Float.isInfinite(fundamentalHz) || Float.isNaN(fundamentalHz)) ? 220.0f : RangesKt.coerceIn(fundamentalHz, 20.0f, 4000.0f);
            float coerceAtMost = RangesKt.coerceAtMost(sanitizeUnit(masterGain), 0.92f);
            float[] fArr = new float[16];
            for (int i = 0; i < 16; i++) {
                fArr[i] = AdditiveSynthState.INSTANCE.sanitizeUnit(partialGains.get(i).floatValue());
            }
            return new AdditiveSynthState(coerceIn, coerceAtMost, fArr, playing, null);
        }

        /* JADX INFO: Access modifiers changed from: private */
        public final float sanitizeUnit(float value) {
            if (Float.isInfinite(value) || Float.isNaN(value)) {
                return 0.0f;
            }
            return RangesKt.coerceIn(value, 0.0f, 1.0f);
        }
    }
}

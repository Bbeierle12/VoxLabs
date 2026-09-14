package org.vocaltract.pixel;

import java.util.Arrays;
import kotlin.Metadata;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: SharedLabState.kt */
@Metadata(d1 = {"\u0000V\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0005\n\u0002\u0010\u0014\n\u0002\b\u0003\n\u0002\u0010\u0007\n\u0002\b\t\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0010\u000e\n\u0002\b\u0003\n\u0002\u0010\t\n\u0002\b\u0004\n\u0002\u0010\u0002\n\u0000\n\u0002\u0010\b\n\u0002\b\u0004\n\u0002\u0018\u0002\n\u0002\b\r\n\u0002\u0018\u0002\n\u0000\u0018\u0000 :2\u00020\u0001:\u0001:B\u000f\u0012\u0006\u0010\u0002\u001a\u00020\u0003¢\u0006\u0004\b\u0004\u0010\u0005J\u0016\u0010#\u001a\u00020$2\u0006\u0010%\u001a\u00020&2\u0006\u0010\b\u001a\u00020\rJ\u000e\u0010'\u001a\u00020$2\u0006\u0010(\u001a\u00020\tJ'\u0010)\u001a\u00020$2\u0006\u0010*\u001a\u00020+2\b\u0010,\u001a\u0004\u0018\u00010\r2\b\b\u0002\u0010-\u001a\u00020\u001b¢\u0006\u0002\u0010.J\u000e\u0010/\u001a\u00020$2\u0006\u0010\b\u001a\u00020\rJ\u000e\u00100\u001a\u00020$2\u0006\u0010\b\u001a\u00020\rJ\u0016\u00101\u001a\u00020$2\u0006\u0010%\u001a\u00020&2\u0006\u0010\b\u001a\u00020\rJ\u0006\u00102\u001a\u00020$J\u0006\u00103\u001a\u00020$J&\u00104\u001a\u00020$2\u0006\u00105\u001a\u00020\t2\u0006\u0010,\u001a\u00020\r2\u0006\u00106\u001a\u00020\r2\u0006\u00107\u001a\u00020\tJ\u0006\u00108\u001a\u000209R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0006\u0010\u0007R\u001e\u0010\n\u001a\u00020\t2\u0006\u0010\b\u001a\u00020\t@BX\u0086\u000e¢\u0006\b\n\u0000\u001a\u0004\b\u000b\u0010\fR\u001e\u0010\u000e\u001a\u00020\r2\u0006\u0010\b\u001a\u00020\r@BX\u0086\u000e¢\u0006\b\n\u0000\u001a\u0004\b\u000f\u0010\u0010R\u001e\u0010\u0011\u001a\u00020\r2\u0006\u0010\b\u001a\u00020\r@BX\u0086\u000e¢\u0006\b\n\u0000\u001a\u0004\b\u0012\u0010\u0010R\u001e\u0010\u0013\u001a\u00020\t2\u0006\u0010\b\u001a\u00020\t@BX\u0086\u000e¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\fR\"\u0010\u0015\u001a\u0004\u0018\u00010\t2\b\u0010\b\u001a\u0004\u0018\u00010\t@BX\u0086\u000e¢\u0006\b\n\u0000\u001a\u0004\b\u0016\u0010\fR\"\u0010\u0018\u001a\u0004\u0018\u00010\u00172\b\u0010\b\u001a\u0004\u0018\u00010\u0017@BX\u0086\u000e¢\u0006\b\n\u0000\u001a\u0004\b\u0019\u0010\u001aR\u001e\u0010\u001c\u001a\u00020\u001b2\u0006\u0010\b\u001a\u00020\u001b@BX\u0086\u000e¢\u0006\b\n\u0000\u001a\u0004\b\u001d\u0010\u001eR\u001e\u0010 \u001a\u00020\u001f2\u0006\u0010\b\u001a\u00020\u001f@BX\u0086\u000e¢\u0006\b\n\u0000\u001a\u0004\b!\u0010\""}, d2 = {"Lorg/vocaltract/pixel/SharedLabState;", "", "model", "Lorg/vocaltract/pixel/SharedTractModel;", "<init>", "(Lorg/vocaltract/pixel/SharedTractModel;)V", "getModel", "()Lorg/vocaltract/pixel/SharedTractModel;", "value", "", "coefficients", "getCoefficients", "()[F", "", "f0", "getF0", "()F", "master", "getMaster", "sourceGains", "getSourceGains", "targets", "getTargets", "Lorg/vocaltract/pixel/TractFit;", "fit", "getFit", "()Lorg/vocaltract/pixel/TractFit;", "", "origin", "getOrigin", "()Ljava/lang/String;", "", "revision", "getRevision", "()J", "setMode", "", "index", "", "tune", "values", "acceptLive", "posterior", "Lorg/vocaltract/pixel/TractPosterior;", "pitch", "sourceLabel", "(Lorg/vocaltract/pixel/TractPosterior;Ljava/lang/Float;Ljava/lang/String;)V", "setPitch", "setMaster", "setPartial", "resetSource", "reset", "restore", "c", "level", "gains", "sound", "Lorg/vocaltract/pixel/AdditiveSynthState;", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class SharedLabState {

    /* renamed from: Companion, reason: from kotlin metadata */
    public static final Companion INSTANCE = new Companion(null);
    private float[] coefficients;
    private float f0;
    private TractFit fit;
    private float master;
    private final SharedTractModel model;
    private String origin;
    private long revision;
    private float[] sourceGains;
    private float[] targets;

    public SharedLabState(SharedTractModel model) {
        Intrinsics.checkNotNullParameter(model, "model");
        this.model = model;
        this.coefficients = new float[4];
        this.f0 = 165.0f;
        this.master = 0.25f;
        this.sourceGains = INSTANCE.defaults();
        this.origin = "MRI mean";
    }

    public final SharedTractModel getModel() {
        return this.model;
    }

    public final float[] getCoefficients() {
        return this.coefficients;
    }

    public final float getF0() {
        return this.f0;
    }

    public final float getMaster() {
        return this.master;
    }

    public final float[] getSourceGains() {
        return this.sourceGains;
    }

    public final float[] getTargets() {
        return this.targets;
    }

    public final TractFit getFit() {
        return this.fit;
    }

    public final String getOrigin() {
        return this.origin;
    }

    public final long getRevision() {
        return this.revision;
    }

    public final void setMode(int index, float value) {
        if (index < 0 || index >= 4) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        float[] fArr = this.coefficients;
        float[] copyOf = Arrays.copyOf(fArr, fArr.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        copyOf[index] = value;
        this.coefficients = this.model.checked(copyOf);
        this.targets = null;
        this.fit = null;
        this.origin = "Manual MRI coordinates";
        this.revision++;
    }

    public final void tune(float[] values) {
        Intrinsics.checkNotNullParameter(values, "values");
        TractFit fit$default = SharedTractModel.fit$default(this.model, values, this.coefficients, 0, 4, null);
        float[] coefficients = fit$default.getCoefficients();
        float[] copyOf = Arrays.copyOf(coefficients, coefficients.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        this.coefficients = copyOf;
        float[] copyOf2 = Arrays.copyOf(values, values.length);
        Intrinsics.checkNotNullExpressionValue(copyOf2, "copyOf(...)");
        this.targets = copyOf2;
        this.fit = fit$default;
        this.origin = "Formant target fit";
        this.revision++;
    }

    public static /* synthetic */ void acceptLive$default(SharedLabState sharedLabState, TractPosterior tractPosterior, Float f, String str, int i, Object obj) {
        if ((i & 4) != 0) {
            str = "Microphone";
        }
        sharedLabState.acceptLive(tractPosterior, f, str);
    }

    public final void acceptLive(TractPosterior posterior, Float pitch, String sourceLabel) {
        StringBuilder sb;
        String str;
        Intrinsics.checkNotNullParameter(posterior, "posterior");
        Intrinsics.checkNotNullParameter(sourceLabel, "sourceLabel");
        this.coefficients = this.model.checked(posterior.getCoefficients());
        if (pitch != null) {
            float floatValue = pitch.floatValue();
            if (!Float.isInfinite(floatValue) && !Float.isNaN(floatValue)) {
                this.f0 = RangesKt.coerceIn(pitch.floatValue(), 55.0f, 800.0f);
            }
        }
        this.targets = null;
        this.fit = null;
        if (posterior.getAbstained()) {
            sb = new StringBuilder();
            sb.append(sourceLabel);
            str = " uncertain · posterior decays toward mean";
        } else {
            sb = new StringBuilder();
            sb.append(sourceLabel);
            str = " estimate";
        }
        sb.append(str);
        this.origin = sb.toString();
        this.revision++;
    }

    public final void setPitch(float value) {
        if (Float.isInfinite(value) || Float.isNaN(value) || 55.0f > value || value > 800.0f) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        this.f0 = value;
        this.revision++;
    }

    public final void setMaster(float value) {
        if (Float.isInfinite(value) || Float.isNaN(value) || 0.0f > value || value > 0.92f) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        this.master = value;
        this.revision++;
    }

    public final void setPartial(int index, float value) {
        if (index < 0 || index >= 16 || Float.isInfinite(value) || Float.isNaN(value) || 0.0f > value || value > 1.0f) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        float[] fArr = this.sourceGains;
        float[] copyOf = Arrays.copyOf(fArr, fArr.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        copyOf[index] = value;
        this.sourceGains = copyOf;
        this.revision++;
    }

    public final void resetSource() {
        this.sourceGains = INSTANCE.defaults();
        this.revision++;
    }

    public final void reset() {
        this.coefficients = new float[4];
        this.f0 = 165.0f;
        this.master = 0.25f;
        this.sourceGains = INSTANCE.defaults();
        this.targets = null;
        this.fit = null;
        this.origin = "MRI mean";
        this.revision++;
    }

    public final void restore(float[] c, float pitch, float level, float[] gains) {
        int i;
        Intrinsics.checkNotNullParameter(c, "c");
        Intrinsics.checkNotNullParameter(gains, "gains");
        float[] checked = this.model.checked(c);
        if (Float.isInfinite(pitch) || Float.isNaN(pitch) || 55.0f > pitch || pitch > 800.0f || Float.isInfinite(level) || Float.isNaN(level) || 0.0f > level || level > 0.92f) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (gains.length == 16) {
            int length = gains.length;
            while (i < length) {
                float f = gains[i];
                i = (!Float.isInfinite(f) && !Float.isNaN(f) && 0.0f <= f && f <= 1.0f) ? i + 1 : 0;
            }
            this.coefficients = checked;
            this.f0 = pitch;
            this.master = level;
            float[] copyOf = Arrays.copyOf(gains, gains.length);
            Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
            this.sourceGains = copyOf;
            this.targets = null;
            this.fit = null;
            this.origin = "Saved MRI coordinates";
            this.revision++;
            return;
        }
        throw new IllegalArgumentException("Failed requirement.".toString());
    }

    /* compiled from: SharedLabState.kt */
    @Metadata(d1 = {"\u0000\u0010\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\u0014\b\u0086\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u0006\u0010\u0004\u001a\u00020\u0005"}, d2 = {"Lorg/vocaltract/pixel/SharedLabState$Companion;", "", "<init>", "()V", "defaults", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
    public static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }

        public final float[] defaults() {
            float[] fArr = new float[16];
            for (int i = 0; i < 16; i++) {
                fArr[i] = (float) Math.pow(i + 1.0d, -0.62d);
            }
            return fArr;
        }
    }

    public final AdditiveSynthState sound() {
        return this.model.sound(this.coefficients, this.f0, this.sourceGains, this.master);
    }
}

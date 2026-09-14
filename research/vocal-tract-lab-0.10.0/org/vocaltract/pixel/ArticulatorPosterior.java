package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.collections.ArraysKt;
import kotlin.io.ConstantsKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;
import kotlin.uuid.Uuid;

/* compiled from: ArticulatorPosterior.kt */
@Metadata(d1 = {"\u0000*\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u000b\n\u0002\u0010\u000e\n\u0002\b!\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010\b\n\u0002\b\u0002\b\u0086\b\u0018\u0000 62\u00020\u0001:\u00016By\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0003\u0012\u0006\u0010\u0006\u001a\u00020\u0003\u0012\u0006\u0010\u0007\u001a\u00020\u0003\u0012\u0006\u0010\b\u001a\u00020\u0003\u0012\u0006\u0010\t\u001a\u00020\u0003\u0012\n\b\u0002\u0010\n\u001a\u0004\u0018\u00010\u0003\u0012\n\b\u0002\u0010\u000b\u001a\u0004\u0018\u00010\u0003\u0012\n\b\u0002\u0010\f\u001a\u0004\u0018\u00010\u0003\u0012\n\b\u0002\u0010\r\u001a\u0004\u0018\u00010\u0003\u0012\b\b\u0002\u0010\u000e\u001a\u00020\u000f¢\u0006\u0004\b\u0010\u0010\u0011J\t\u0010\"\u001a\u00020\u0003HÆ\u0003J\t\u0010#\u001a\u00020\u0003HÆ\u0003J\t\u0010$\u001a\u00020\u0003HÆ\u0003J\t\u0010%\u001a\u00020\u0003HÆ\u0003J\t\u0010&\u001a\u00020\u0003HÆ\u0003J\t\u0010'\u001a\u00020\u0003HÆ\u0003J\t\u0010(\u001a\u00020\u0003HÆ\u0003J\u0010\u0010)\u001a\u0004\u0018\u00010\u0003HÆ\u0003¢\u0006\u0002\u0010\u001bJ\u0010\u0010*\u001a\u0004\u0018\u00010\u0003HÆ\u0003¢\u0006\u0002\u0010\u001bJ\u0010\u0010+\u001a\u0004\u0018\u00010\u0003HÆ\u0003¢\u0006\u0002\u0010\u001bJ\u0010\u0010,\u001a\u0004\u0018\u00010\u0003HÆ\u0003¢\u0006\u0002\u0010\u001bJ\t\u0010-\u001a\u00020\u000fHÆ\u0003J\u008e\u0001\u0010.\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00032\b\b\u0002\u0010\u0005\u001a\u00020\u00032\b\b\u0002\u0010\u0006\u001a\u00020\u00032\b\b\u0002\u0010\u0007\u001a\u00020\u00032\b\b\u0002\u0010\b\u001a\u00020\u00032\b\b\u0002\u0010\t\u001a\u00020\u00032\n\b\u0002\u0010\n\u001a\u0004\u0018\u00010\u00032\n\b\u0002\u0010\u000b\u001a\u0004\u0018\u00010\u00032\n\b\u0002\u0010\f\u001a\u0004\u0018\u00010\u00032\n\b\u0002\u0010\r\u001a\u0004\u0018\u00010\u00032\b\b\u0002\u0010\u000e\u001a\u00020\u000fHÆ\u0001¢\u0006\u0002\u0010/J\u0013\u00100\u001a\u0002012\b\u00102\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u00103\u001a\u000204HÖ\u0001J\t\u00105\u001a\u00020\u000fHÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0012\u0010\u0013R\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\u0013R\u0011\u0010\u0005\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0015\u0010\u0013R\u0011\u0010\u0006\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0016\u0010\u0013R\u0011\u0010\u0007\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0017\u0010\u0013R\u0011\u0010\b\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0018\u0010\u0013R\u0011\u0010\t\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0019\u0010\u0013R\u0015\u0010\n\u001a\u0004\u0018\u00010\u0003¢\u0006\n\n\u0002\u0010\u001c\u001a\u0004\b\u001a\u0010\u001bR\u0015\u0010\u000b\u001a\u0004\u0018\u00010\u0003¢\u0006\n\n\u0002\u0010\u001c\u001a\u0004\b\u001d\u0010\u001bR\u0015\u0010\f\u001a\u0004\u0018\u00010\u0003¢\u0006\n\n\u0002\u0010\u001c\u001a\u0004\b\u001e\u0010\u001bR\u0015\u0010\r\u001a\u0004\u0018\u00010\u0003¢\u0006\n\n\u0002\u0010\u001c\u001a\u0004\b\u001f\u0010\u001bR\u0011\u0010\u000e\u001a\u00020\u000f¢\u0006\b\n\u0000\u001a\u0004\b \u0010!"}, d2 = {"Lorg/vocaltract/pixel/ArticulatorPosterior;", "", "jawOpening", "", "lipAperture", "tongueFrontBack", "tongueDorsum", "tongueRoot", "pharynxWidth", "epilarynxWidth", "lipProtrusion", "larynxHeight", "velumOpening", "nasalCoupling", "interpretation", "", "<init>", "(FFFFFFFLjava/lang/Float;Ljava/lang/Float;Ljava/lang/Float;Ljava/lang/Float;Ljava/lang/String;)V", "getJawOpening", "()F", "getLipAperture", "getTongueFrontBack", "getTongueDorsum", "getTongueRoot", "getPharynxWidth", "getEpilarynxWidth", "getLipProtrusion", "()Ljava/lang/Float;", "Ljava/lang/Float;", "getLarynxHeight", "getVelumOpening", "getNasalCoupling", "getInterpretation", "()Ljava/lang/String;", "component1", "component2", "component3", "component4", "component5", "component6", "component7", "component8", "component9", "component10", "component11", "component12", "copy", "(FFFFFFFLjava/lang/Float;Ljava/lang/Float;Ljava/lang/Float;Ljava/lang/Float;Ljava/lang/String;)Lorg/vocaltract/pixel/ArticulatorPosterior;", "equals", "", "other", "hashCode", "", "toString", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class ArticulatorPosterior {

    /* renamed from: Companion, reason: from kotlin metadata */
    public static final Companion INSTANCE = new Companion(null);
    private final float epilarynxWidth;
    private final String interpretation;
    private final float jawOpening;
    private final Float larynxHeight;
    private final float lipAperture;
    private final Float lipProtrusion;
    private final Float nasalCoupling;
    private final float pharynxWidth;
    private final float tongueDorsum;
    private final float tongueFrontBack;
    private final float tongueRoot;
    private final Float velumOpening;

    /* renamed from: component1, reason: from getter */
    public final float getJawOpening() {
        return this.jawOpening;
    }

    /* renamed from: component10, reason: from getter */
    public final Float getVelumOpening() {
        return this.velumOpening;
    }

    /* renamed from: component11, reason: from getter */
    public final Float getNasalCoupling() {
        return this.nasalCoupling;
    }

    /* renamed from: component12, reason: from getter */
    public final String getInterpretation() {
        return this.interpretation;
    }

    /* renamed from: component2, reason: from getter */
    public final float getLipAperture() {
        return this.lipAperture;
    }

    /* renamed from: component3, reason: from getter */
    public final float getTongueFrontBack() {
        return this.tongueFrontBack;
    }

    /* renamed from: component4, reason: from getter */
    public final float getTongueDorsum() {
        return this.tongueDorsum;
    }

    /* renamed from: component5, reason: from getter */
    public final float getTongueRoot() {
        return this.tongueRoot;
    }

    /* renamed from: component6, reason: from getter */
    public final float getPharynxWidth() {
        return this.pharynxWidth;
    }

    /* renamed from: component7, reason: from getter */
    public final float getEpilarynxWidth() {
        return this.epilarynxWidth;
    }

    /* renamed from: component8, reason: from getter */
    public final Float getLipProtrusion() {
        return this.lipProtrusion;
    }

    /* renamed from: component9, reason: from getter */
    public final Float getLarynxHeight() {
        return this.larynxHeight;
    }

    public final ArticulatorPosterior copy(float jawOpening, float lipAperture, float tongueFrontBack, float tongueDorsum, float tongueRoot, float pharynxWidth, float epilarynxWidth, Float lipProtrusion, Float larynxHeight, Float velumOpening, Float nasalCoupling, String interpretation) {
        Intrinsics.checkNotNullParameter(interpretation, "interpretation");
        return new ArticulatorPosterior(jawOpening, lipAperture, tongueFrontBack, tongueDorsum, tongueRoot, pharynxWidth, epilarynxWidth, lipProtrusion, larynxHeight, velumOpening, nasalCoupling, interpretation);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof ArticulatorPosterior)) {
            return false;
        }
        ArticulatorPosterior articulatorPosterior = (ArticulatorPosterior) other;
        return Float.compare(this.jawOpening, articulatorPosterior.jawOpening) == 0 && Float.compare(this.lipAperture, articulatorPosterior.lipAperture) == 0 && Float.compare(this.tongueFrontBack, articulatorPosterior.tongueFrontBack) == 0 && Float.compare(this.tongueDorsum, articulatorPosterior.tongueDorsum) == 0 && Float.compare(this.tongueRoot, articulatorPosterior.tongueRoot) == 0 && Float.compare(this.pharynxWidth, articulatorPosterior.pharynxWidth) == 0 && Float.compare(this.epilarynxWidth, articulatorPosterior.epilarynxWidth) == 0 && Intrinsics.areEqual((Object) this.lipProtrusion, (Object) articulatorPosterior.lipProtrusion) && Intrinsics.areEqual((Object) this.larynxHeight, (Object) articulatorPosterior.larynxHeight) && Intrinsics.areEqual((Object) this.velumOpening, (Object) articulatorPosterior.velumOpening) && Intrinsics.areEqual((Object) this.nasalCoupling, (Object) articulatorPosterior.nasalCoupling) && Intrinsics.areEqual(this.interpretation, articulatorPosterior.interpretation);
    }

    public int hashCode() {
        int hashCode = ((((((((((((Float.hashCode(this.jawOpening) * 31) + Float.hashCode(this.lipAperture)) * 31) + Float.hashCode(this.tongueFrontBack)) * 31) + Float.hashCode(this.tongueDorsum)) * 31) + Float.hashCode(this.tongueRoot)) * 31) + Float.hashCode(this.pharynxWidth)) * 31) + Float.hashCode(this.epilarynxWidth)) * 31;
        Float f = this.lipProtrusion;
        int hashCode2 = (hashCode + (f == null ? 0 : f.hashCode())) * 31;
        Float f2 = this.larynxHeight;
        int hashCode3 = (hashCode2 + (f2 == null ? 0 : f2.hashCode())) * 31;
        Float f3 = this.velumOpening;
        int hashCode4 = (hashCode3 + (f3 == null ? 0 : f3.hashCode())) * 31;
        Float f4 = this.nasalCoupling;
        return ((hashCode4 + (f4 != null ? f4.hashCode() : 0)) * 31) + this.interpretation.hashCode();
    }

    public String toString() {
        return "ArticulatorPosterior(jawOpening=" + this.jawOpening + ", lipAperture=" + this.lipAperture + ", tongueFrontBack=" + this.tongueFrontBack + ", tongueDorsum=" + this.tongueDorsum + ", tongueRoot=" + this.tongueRoot + ", pharynxWidth=" + this.pharynxWidth + ", epilarynxWidth=" + this.epilarynxWidth + ", lipProtrusion=" + this.lipProtrusion + ", larynxHeight=" + this.larynxHeight + ", velumOpening=" + this.velumOpening + ", nasalCoupling=" + this.nasalCoupling + ", interpretation=" + this.interpretation + ")";
    }

    public ArticulatorPosterior(float f, float f2, float f3, float f4, float f5, float f6, float f7, Float f8, Float f9, Float f10, Float f11, String interpretation) {
        Intrinsics.checkNotNullParameter(interpretation, "interpretation");
        this.jawOpening = f;
        this.lipAperture = f2;
        this.tongueFrontBack = f3;
        this.tongueDorsum = f4;
        this.tongueRoot = f5;
        this.pharynxWidth = f6;
        this.epilarynxWidth = f7;
        this.lipProtrusion = f8;
        this.larynxHeight = f9;
        this.velumOpening = f10;
        this.nasalCoupling = f11;
        this.interpretation = interpretation;
    }

    public final float getJawOpening() {
        return this.jawOpening;
    }

    public final float getLipAperture() {
        return this.lipAperture;
    }

    public final float getTongueFrontBack() {
        return this.tongueFrontBack;
    }

    public final float getTongueDorsum() {
        return this.tongueDorsum;
    }

    public final float getTongueRoot() {
        return this.tongueRoot;
    }

    public final float getPharynxWidth() {
        return this.pharynxWidth;
    }

    public final float getEpilarynxWidth() {
        return this.epilarynxWidth;
    }

    public final Float getLipProtrusion() {
        return this.lipProtrusion;
    }

    public final Float getLarynxHeight() {
        return this.larynxHeight;
    }

    public final Float getVelumOpening() {
        return this.velumOpening;
    }

    public final Float getNasalCoupling() {
        return this.nasalCoupling;
    }

    public final String getInterpretation() {
        return this.interpretation;
    }

    public /* synthetic */ ArticulatorPosterior(float f, float f2, float f3, float f4, float f5, float f6, float f7, Float f8, Float f9, Float f10, Float f11, String str, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(f, f2, f3, f4, f5, f6, f7, (i & Uuid.SIZE_BITS) != 0 ? null : f8, (i & 256) != 0 ? null : f9, (i & ConstantsKt.MINIMUM_BLOCK_SIZE) != 0 ? null : f10, (i & 1024) != 0 ? null : f11, (i & 2048) != 0 ? "Bounded area-derived animation proxies; unavailable controls are not inferred" : str);
    }

    /* compiled from: ArticulatorPosterior.kt */
    @Metadata(d1 = {"\u0000\u001a\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0010\u0014\n\u0000\b\u0086\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u0006\u0010\u0004\u001a\u00020\u0005J\u0016\u0010\u0006\u001a\u00020\u00052\u0006\u0010\u0007\u001a\u00020\b2\u0006\u0010\t\u001a\u00020\b"}, d2 = {"Lorg/vocaltract/pixel/ArticulatorPosterior$Companion;", "", "<init>", "()V", "neutral", "Lorg/vocaltract/pixel/ArticulatorPosterior;", "infer", "area", "", "mean"}, k = 1, mv = {2, 0, 0}, xi = 48)
    public static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }

        public final ArticulatorPosterior neutral() {
            return new ArticulatorPosterior(0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, null, null, null, null, null, 3968, null);
        }

        public final ArticulatorPosterior infer(float[] area, float[] mean) {
            Intrinsics.checkNotNullParameter(area, "area");
            Intrinsics.checkNotNullParameter(mean, "mean");
            if (area.length != mean.length || area.length < 16) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            float infer$regional = infer$regional(area, mean, 0.0f, 0.16f);
            float infer$regional2 = infer$regional(area, mean, 0.16f, 0.45f);
            float infer$regional3 = infer$regional(area, mean, 0.28f, 0.52f);
            float infer$regional4 = infer$regional(area, mean, 0.48f, 0.72f);
            float infer$regional5 = infer$regional(area, mean, 0.68f, 0.88f);
            float infer$regional6 = infer$regional(area, mean, 0.88f, 1.0f);
            return new ArticulatorPosterior(RangesKt.coerceIn((0.55f * infer$regional6) + (0.45f * infer$regional5), -1.0f, 1.0f), infer$regional6, RangesKt.coerceIn(infer$regional5 - infer$regional3, -1.0f, 1.0f), RangesKt.coerceIn(-infer$regional4, -1.0f, 1.0f), RangesKt.coerceIn(-infer$regional3, -1.0f, 1.0f), infer$regional2, infer$regional, null, null, null, null, null, 3968, null);
        }

        private static final float infer$regional(float[] fArr, float[] fArr2, float f, float f2) {
            int coerceIn = RangesKt.coerceIn((int) (f * fArr.length), 0, ArraysKt.getLastIndex(fArr));
            int coerceIn2 = RangesKt.coerceIn((int) (f2 * fArr.length), coerceIn + 1, fArr.length);
            float f3 = 0.0f;
            for (int i = coerceIn; i < coerceIn2; i++) {
                f3 += (float) Math.log(RangesKt.coerceAtLeast(fArr[i] / fArr2[i], 1.0E-4f));
            }
            return RangesKt.coerceIn(f3 / (coerceIn2 - coerceIn), -1.0f, 1.0f);
        }
    }
}

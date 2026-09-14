package org.vocaltract.pixel;

import android.content.Context;
import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.Reader;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.Iterator;
import java.util.List;
import kotlin.Metadata;
import kotlin.Pair;
import kotlin.collections.IntIterator;
import kotlin.io.CloseableKt;
import kotlin.io.ConstantsKt;
import kotlin.io.TextStreamsKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;
import kotlin.text.Charsets;
import org.json.JSONArray;
import org.json.JSONObject;

/* compiled from: ReducedModelAsset.kt */
@Metadata(d1 = {"\u0000F\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u000e\n\u0002\b\u0004\n\u0002\u0010\u0014\n\u0002\b\u0002\n\u0002\u0010\u0011\n\u0002\b\u0005\n\u0002\u0010 \n\u0002\b\u0005\n\u0002\u0010\u000b\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u001e\n\u0002\u0018\u0002\n\u0002\b\u001d\b\u0086\b\u0018\u0000 W2\u00020\u0001:\u0001WBË\u0001\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0003\u0012\u0006\u0010\u0007\u001a\u00020\u0003\u0012\b\b\u0002\u0010\b\u001a\u00020\u0003\u0012\u0006\u0010\t\u001a\u00020\n\u0012\u0006\u0010\u000b\u001a\u00020\n\u0012\f\u0010\f\u001a\b\u0012\u0004\u0012\u00020\n0\r\u0012\u0006\u0010\u000e\u001a\u00020\n\u0012\u0006\u0010\u000f\u001a\u00020\n\u0012\u000e\b\u0002\u0010\u0010\u001a\b\u0012\u0004\u0012\u00020\n0\r\u0012\u000e\b\u0002\u0010\u0011\u001a\b\u0012\u0004\u0012\u00020\n0\r\u0012\u000e\b\u0002\u0010\u0012\u001a\b\u0012\u0004\u0012\u00020\u00050\u0013\u0012\b\b\u0002\u0010\u0014\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0015\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0016\u001a\u00020\u0003\u0012\b\b\u0002\u0010\u0017\u001a\u00020\u0003\u0012\b\b\u0002\u0010\u0018\u001a\u00020\u0019\u0012\b\b\u0002\u0010\u001a\u001a\u00020\u001b¢\u0006\u0004\b\u001c\u0010\u001dJ \u00109\u001a\u000e\u0012\u0004\u0012\u00020\n\u0012\u0004\u0012\u00020\n0:2\f\u0010;\u001a\b\u0012\u0004\u0012\u00020\u001b0\u0013J\u000e\u0010<\u001a\u00020\n2\u0006\u0010=\u001a\u00020\nJ\t\u0010>\u001a\u00020\u0003HÆ\u0003J\t\u0010?\u001a\u00020\u0005HÆ\u0003J\t\u0010@\u001a\u00020\u0003HÆ\u0003J\t\u0010A\u001a\u00020\u0003HÆ\u0003J\t\u0010B\u001a\u00020\u0003HÆ\u0003J\t\u0010C\u001a\u00020\nHÆ\u0003J\t\u0010D\u001a\u00020\nHÆ\u0003J\u0014\u0010E\u001a\b\u0012\u0004\u0012\u00020\n0\rHÆ\u0003¢\u0006\u0002\u0010)J\t\u0010F\u001a\u00020\nHÆ\u0003J\t\u0010G\u001a\u00020\nHÆ\u0003J\u0014\u0010H\u001a\b\u0012\u0004\u0012\u00020\n0\rHÆ\u0003¢\u0006\u0002\u0010)J\u0014\u0010I\u001a\b\u0012\u0004\u0012\u00020\n0\rHÆ\u0003¢\u0006\u0002\u0010)J\u000f\u0010J\u001a\b\u0012\u0004\u0012\u00020\u00050\u0013HÆ\u0003J\t\u0010K\u001a\u00020\u0005HÆ\u0003J\t\u0010L\u001a\u00020\u0005HÆ\u0003J\t\u0010M\u001a\u00020\u0003HÆ\u0003J\t\u0010N\u001a\u00020\u0003HÆ\u0003J\t\u0010O\u001a\u00020\u0019HÆ\u0003J\t\u0010P\u001a\u00020\u001bHÆ\u0003Jä\u0001\u0010Q\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00032\b\b\u0002\u0010\u0007\u001a\u00020\u00032\b\b\u0002\u0010\b\u001a\u00020\u00032\b\b\u0002\u0010\t\u001a\u00020\n2\b\b\u0002\u0010\u000b\u001a\u00020\n2\u000e\b\u0002\u0010\f\u001a\b\u0012\u0004\u0012\u00020\n0\r2\b\b\u0002\u0010\u000e\u001a\u00020\n2\b\b\u0002\u0010\u000f\u001a\u00020\n2\u000e\b\u0002\u0010\u0010\u001a\b\u0012\u0004\u0012\u00020\n0\r2\u000e\b\u0002\u0010\u0011\u001a\b\u0012\u0004\u0012\u00020\n0\r2\u000e\b\u0002\u0010\u0012\u001a\b\u0012\u0004\u0012\u00020\u00050\u00132\b\b\u0002\u0010\u0014\u001a\u00020\u00052\b\b\u0002\u0010\u0015\u001a\u00020\u00052\b\b\u0002\u0010\u0016\u001a\u00020\u00032\b\b\u0002\u0010\u0017\u001a\u00020\u00032\b\b\u0002\u0010\u0018\u001a\u00020\u00192\b\b\u0002\u0010\u001a\u001a\u00020\u001bHÆ\u0001¢\u0006\u0002\u0010RJ\u0013\u0010S\u001a\u00020\u00192\b\u0010T\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010U\u001a\u00020\u0003HÖ\u0001J\t\u0010V\u001a\u00020\u0005HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u001e\u0010\u001fR\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b \u0010!R\u0011\u0010\u0006\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\"\u0010\u001fR\u0011\u0010\u0007\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b#\u0010\u001fR\u0011\u0010\b\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b$\u0010\u001fR\u0011\u0010\t\u001a\u00020\n¢\u0006\b\n\u0000\u001a\u0004\b%\u0010&R\u0011\u0010\u000b\u001a\u00020\n¢\u0006\b\n\u0000\u001a\u0004\b'\u0010&R\u0019\u0010\f\u001a\b\u0012\u0004\u0012\u00020\n0\r¢\u0006\n\n\u0002\u0010*\u001a\u0004\b(\u0010)R\u0011\u0010\u000e\u001a\u00020\n¢\u0006\b\n\u0000\u001a\u0004\b+\u0010&R\u0011\u0010\u000f\u001a\u00020\n¢\u0006\b\n\u0000\u001a\u0004\b,\u0010&R\u0019\u0010\u0010\u001a\b\u0012\u0004\u0012\u00020\n0\r¢\u0006\n\n\u0002\u0010*\u001a\u0004\b-\u0010)R\u0019\u0010\u0011\u001a\b\u0012\u0004\u0012\u00020\n0\r¢\u0006\n\n\u0002\u0010*\u001a\u0004\b.\u0010)R\u0017\u0010\u0012\u001a\b\u0012\u0004\u0012\u00020\u00050\u0013¢\u0006\b\n\u0000\u001a\u0004\b/\u00100R\u0011\u0010\u0014\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b1\u0010!R\u0011\u0010\u0015\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b2\u0010!R\u0011\u0010\u0016\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b3\u0010\u001fR\u0011\u0010\u0017\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b4\u0010\u001fR\u0011\u0010\u0018\u001a\u00020\u0019¢\u0006\b\n\u0000\u001a\u0004\b5\u00106R\u0011\u0010\u001a\u001a\u00020\u001b¢\u0006\b\n\u0000\u001a\u0004\b7\u00108"}, d2 = {"Lorg/vocaltract/pixel/ReducedModelAsset;", "", "schemaVersion", "", "modelId", "", "sampleRateHz", "frameSize", "hopSize", "sectionPosition", "", "areaMeanCm2", "areaModes", "", "referenceFormantsHz", "coefficientScaleHz", "formantToMode", "coefficientLimitsSd", "modeLabels", "", "atlasSourceModelSha256", "atlasFreezeManifestSha256", "atlasSubjects", "independentExpertAcceptances", "scientificReleaseReady", "", "abstentionConfidenceThreshold", "", "<init>", "(ILjava/lang/String;III[F[F[[F[F[F[[F[[FLjava/util/List;Ljava/lang/String;Ljava/lang/String;IIZF)V", "getSchemaVersion", "()I", "getModelId", "()Ljava/lang/String;", "getSampleRateHz", "getFrameSize", "getHopSize", "getSectionPosition", "()[F", "getAreaMeanCm2", "getAreaModes", "()[[F", "[[F", "getReferenceFormantsHz", "getCoefficientScaleHz", "getFormantToMode", "getCoefficientLimitsSd", "getModeLabels", "()Ljava/util/List;", "getAtlasSourceModelSha256", "getAtlasFreezeManifestSha256", "getAtlasSubjects", "getIndependentExpertAcceptances", "getScientificReleaseReady", "()Z", "getAbstentionConfidenceThreshold", "()F", "inferArea", "Lkotlin/Pair;", "formantsHz", "areaFromCoefficients", "input", "component1", "component2", "component3", "component4", "component5", "component6", "component7", "component8", "component9", "component10", "component11", "component12", "component13", "component14", "component15", "component16", "component17", "component18", "component19", "copy", "(ILjava/lang/String;III[F[F[[F[F[F[[F[[FLjava/util/List;Ljava/lang/String;Ljava/lang/String;IIZF)Lorg/vocaltract/pixel/ReducedModelAsset;", "equals", "other", "hashCode", "toString", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class ReducedModelAsset {

    /* renamed from: Companion, reason: from kotlin metadata */
    public static final Companion INSTANCE = new Companion(null);
    private final float abstentionConfidenceThreshold;
    private final float[] areaMeanCm2;
    private final float[][] areaModes;
    private final String atlasFreezeManifestSha256;
    private final String atlasSourceModelSha256;
    private final int atlasSubjects;
    private final float[][] coefficientLimitsSd;
    private final float[] coefficientScaleHz;
    private final float[][] formantToMode;
    private final int frameSize;
    private final int hopSize;
    private final int independentExpertAcceptances;
    private final List<String> modeLabels;
    private final String modelId;
    private final float[] referenceFormantsHz;
    private final int sampleRateHz;
    private final int schemaVersion;
    private final boolean scientificReleaseReady;
    private final float[] sectionPosition;

    /* renamed from: component1, reason: from getter */
    public final int getSchemaVersion() {
        return this.schemaVersion;
    }

    /* renamed from: component10, reason: from getter */
    public final float[] getCoefficientScaleHz() {
        return this.coefficientScaleHz;
    }

    /* renamed from: component11, reason: from getter */
    public final float[][] getFormantToMode() {
        return this.formantToMode;
    }

    /* renamed from: component12, reason: from getter */
    public final float[][] getCoefficientLimitsSd() {
        return this.coefficientLimitsSd;
    }

    public final List<String> component13() {
        return this.modeLabels;
    }

    /* renamed from: component14, reason: from getter */
    public final String getAtlasSourceModelSha256() {
        return this.atlasSourceModelSha256;
    }

    /* renamed from: component15, reason: from getter */
    public final String getAtlasFreezeManifestSha256() {
        return this.atlasFreezeManifestSha256;
    }

    /* renamed from: component16, reason: from getter */
    public final int getAtlasSubjects() {
        return this.atlasSubjects;
    }

    /* renamed from: component17, reason: from getter */
    public final int getIndependentExpertAcceptances() {
        return this.independentExpertAcceptances;
    }

    /* renamed from: component18, reason: from getter */
    public final boolean getScientificReleaseReady() {
        return this.scientificReleaseReady;
    }

    /* renamed from: component19, reason: from getter */
    public final float getAbstentionConfidenceThreshold() {
        return this.abstentionConfidenceThreshold;
    }

    /* renamed from: component2, reason: from getter */
    public final String getModelId() {
        return this.modelId;
    }

    /* renamed from: component3, reason: from getter */
    public final int getSampleRateHz() {
        return this.sampleRateHz;
    }

    /* renamed from: component4, reason: from getter */
    public final int getFrameSize() {
        return this.frameSize;
    }

    /* renamed from: component5, reason: from getter */
    public final int getHopSize() {
        return this.hopSize;
    }

    /* renamed from: component6, reason: from getter */
    public final float[] getSectionPosition() {
        return this.sectionPosition;
    }

    /* renamed from: component7, reason: from getter */
    public final float[] getAreaMeanCm2() {
        return this.areaMeanCm2;
    }

    /* renamed from: component8, reason: from getter */
    public final float[][] getAreaModes() {
        return this.areaModes;
    }

    /* renamed from: component9, reason: from getter */
    public final float[] getReferenceFormantsHz() {
        return this.referenceFormantsHz;
    }

    public final ReducedModelAsset copy(int schemaVersion, String modelId, int sampleRateHz, int frameSize, int hopSize, float[] sectionPosition, float[] areaMeanCm2, float[][] areaModes, float[] referenceFormantsHz, float[] coefficientScaleHz, float[][] formantToMode, float[][] coefficientLimitsSd, List<String> modeLabels, String atlasSourceModelSha256, String atlasFreezeManifestSha256, int atlasSubjects, int independentExpertAcceptances, boolean scientificReleaseReady, float abstentionConfidenceThreshold) {
        Intrinsics.checkNotNullParameter(modelId, "modelId");
        Intrinsics.checkNotNullParameter(sectionPosition, "sectionPosition");
        Intrinsics.checkNotNullParameter(areaMeanCm2, "areaMeanCm2");
        Intrinsics.checkNotNullParameter(areaModes, "areaModes");
        Intrinsics.checkNotNullParameter(referenceFormantsHz, "referenceFormantsHz");
        Intrinsics.checkNotNullParameter(coefficientScaleHz, "coefficientScaleHz");
        Intrinsics.checkNotNullParameter(formantToMode, "formantToMode");
        Intrinsics.checkNotNullParameter(coefficientLimitsSd, "coefficientLimitsSd");
        Intrinsics.checkNotNullParameter(modeLabels, "modeLabels");
        Intrinsics.checkNotNullParameter(atlasSourceModelSha256, "atlasSourceModelSha256");
        Intrinsics.checkNotNullParameter(atlasFreezeManifestSha256, "atlasFreezeManifestSha256");
        return new ReducedModelAsset(schemaVersion, modelId, sampleRateHz, frameSize, hopSize, sectionPosition, areaMeanCm2, areaModes, referenceFormantsHz, coefficientScaleHz, formantToMode, coefficientLimitsSd, modeLabels, atlasSourceModelSha256, atlasFreezeManifestSha256, atlasSubjects, independentExpertAcceptances, scientificReleaseReady, abstentionConfidenceThreshold);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof ReducedModelAsset)) {
            return false;
        }
        ReducedModelAsset reducedModelAsset = (ReducedModelAsset) other;
        return this.schemaVersion == reducedModelAsset.schemaVersion && Intrinsics.areEqual(this.modelId, reducedModelAsset.modelId) && this.sampleRateHz == reducedModelAsset.sampleRateHz && this.frameSize == reducedModelAsset.frameSize && this.hopSize == reducedModelAsset.hopSize && Intrinsics.areEqual(this.sectionPosition, reducedModelAsset.sectionPosition) && Intrinsics.areEqual(this.areaMeanCm2, reducedModelAsset.areaMeanCm2) && Intrinsics.areEqual(this.areaModes, reducedModelAsset.areaModes) && Intrinsics.areEqual(this.referenceFormantsHz, reducedModelAsset.referenceFormantsHz) && Intrinsics.areEqual(this.coefficientScaleHz, reducedModelAsset.coefficientScaleHz) && Intrinsics.areEqual(this.formantToMode, reducedModelAsset.formantToMode) && Intrinsics.areEqual(this.coefficientLimitsSd, reducedModelAsset.coefficientLimitsSd) && Intrinsics.areEqual(this.modeLabels, reducedModelAsset.modeLabels) && Intrinsics.areEqual(this.atlasSourceModelSha256, reducedModelAsset.atlasSourceModelSha256) && Intrinsics.areEqual(this.atlasFreezeManifestSha256, reducedModelAsset.atlasFreezeManifestSha256) && this.atlasSubjects == reducedModelAsset.atlasSubjects && this.independentExpertAcceptances == reducedModelAsset.independentExpertAcceptances && this.scientificReleaseReady == reducedModelAsset.scientificReleaseReady && Float.compare(this.abstentionConfidenceThreshold, reducedModelAsset.abstentionConfidenceThreshold) == 0;
    }

    public int hashCode() {
        return (((((((((((((((((((((((((((((((((((Integer.hashCode(this.schemaVersion) * 31) + this.modelId.hashCode()) * 31) + Integer.hashCode(this.sampleRateHz)) * 31) + Integer.hashCode(this.frameSize)) * 31) + Integer.hashCode(this.hopSize)) * 31) + Arrays.hashCode(this.sectionPosition)) * 31) + Arrays.hashCode(this.areaMeanCm2)) * 31) + Arrays.hashCode(this.areaModes)) * 31) + Arrays.hashCode(this.referenceFormantsHz)) * 31) + Arrays.hashCode(this.coefficientScaleHz)) * 31) + Arrays.hashCode(this.formantToMode)) * 31) + Arrays.hashCode(this.coefficientLimitsSd)) * 31) + this.modeLabels.hashCode()) * 31) + this.atlasSourceModelSha256.hashCode()) * 31) + this.atlasFreezeManifestSha256.hashCode()) * 31) + Integer.hashCode(this.atlasSubjects)) * 31) + Integer.hashCode(this.independentExpertAcceptances)) * 31) + Boolean.hashCode(this.scientificReleaseReady)) * 31) + Float.hashCode(this.abstentionConfidenceThreshold);
    }

    public String toString() {
        return "ReducedModelAsset(schemaVersion=" + this.schemaVersion + ", modelId=" + this.modelId + ", sampleRateHz=" + this.sampleRateHz + ", frameSize=" + this.frameSize + ", hopSize=" + this.hopSize + ", sectionPosition=" + Arrays.toString(this.sectionPosition) + ", areaMeanCm2=" + Arrays.toString(this.areaMeanCm2) + ", areaModes=" + Arrays.toString(this.areaModes) + ", referenceFormantsHz=" + Arrays.toString(this.referenceFormantsHz) + ", coefficientScaleHz=" + Arrays.toString(this.coefficientScaleHz) + ", formantToMode=" + Arrays.toString(this.formantToMode) + ", coefficientLimitsSd=" + Arrays.toString(this.coefficientLimitsSd) + ", modeLabels=" + this.modeLabels + ", atlasSourceModelSha256=" + this.atlasSourceModelSha256 + ", atlasFreezeManifestSha256=" + this.atlasFreezeManifestSha256 + ", atlasSubjects=" + this.atlasSubjects + ", independentExpertAcceptances=" + this.independentExpertAcceptances + ", scientificReleaseReady=" + this.scientificReleaseReady + ", abstentionConfidenceThreshold=" + this.abstentionConfidenceThreshold + ")";
    }

    public ReducedModelAsset(int i, String modelId, int i2, int i3, int i4, float[] sectionPosition, float[] areaMeanCm2, float[][] areaModes, float[] referenceFormantsHz, float[] coefficientScaleHz, float[][] formantToMode, float[][] coefficientLimitsSd, List<String> modeLabels, String atlasSourceModelSha256, String atlasFreezeManifestSha256, int i5, int i6, boolean z, float f) {
        Intrinsics.checkNotNullParameter(modelId, "modelId");
        Intrinsics.checkNotNullParameter(sectionPosition, "sectionPosition");
        Intrinsics.checkNotNullParameter(areaMeanCm2, "areaMeanCm2");
        Intrinsics.checkNotNullParameter(areaModes, "areaModes");
        Intrinsics.checkNotNullParameter(referenceFormantsHz, "referenceFormantsHz");
        Intrinsics.checkNotNullParameter(coefficientScaleHz, "coefficientScaleHz");
        Intrinsics.checkNotNullParameter(formantToMode, "formantToMode");
        Intrinsics.checkNotNullParameter(coefficientLimitsSd, "coefficientLimitsSd");
        Intrinsics.checkNotNullParameter(modeLabels, "modeLabels");
        Intrinsics.checkNotNullParameter(atlasSourceModelSha256, "atlasSourceModelSha256");
        Intrinsics.checkNotNullParameter(atlasFreezeManifestSha256, "atlasFreezeManifestSha256");
        this.schemaVersion = i;
        this.modelId = modelId;
        this.sampleRateHz = i2;
        this.frameSize = i3;
        this.hopSize = i4;
        this.sectionPosition = sectionPosition;
        this.areaMeanCm2 = areaMeanCm2;
        this.areaModes = areaModes;
        this.referenceFormantsHz = referenceFormantsHz;
        this.coefficientScaleHz = coefficientScaleHz;
        this.formantToMode = formantToMode;
        this.coefficientLimitsSd = coefficientLimitsSd;
        this.modeLabels = modeLabels;
        this.atlasSourceModelSha256 = atlasSourceModelSha256;
        this.atlasFreezeManifestSha256 = atlasFreezeManifestSha256;
        this.atlasSubjects = i5;
        this.independentExpertAcceptances = i6;
        this.scientificReleaseReady = z;
        this.abstentionConfidenceThreshold = f;
        if (i != 1 && i != 2) {
            throw new IllegalArgumentException(("Unsupported reduced-model schema " + i).toString());
        }
        if (1 > i4 || i4 > i3) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (sectionPosition.length < 16) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (areaMeanCm2.length != sectionPosition.length) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (areaModes.length != referenceFormantsHz.length) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (referenceFormantsHz.length != coefficientScaleHz.length) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        for (float[] fArr : areaModes) {
            if (fArr.length != this.sectionPosition.length) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        for (float f2 : this.areaMeanCm2) {
            if (f2 <= 0.0f || Float.isInfinite(f2) || Float.isNaN(f2)) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        Iterable until = RangesKt.until(1, this.sectionPosition.length);
        if (!(until instanceof Collection) || !((Collection) until).isEmpty()) {
            Iterator it = until.iterator();
            while (it.hasNext()) {
                int nextInt = ((IntIterator) it).nextInt();
                float[] fArr2 = this.sectionPosition;
                if (fArr2[nextInt] <= fArr2[nextInt - 1]) {
                    throw new IllegalArgumentException("Failed requirement.".toString());
                }
            }
        }
        float[][] fArr3 = this.coefficientLimitsSd;
        if (fArr3.length != this.areaModes.length) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        for (float[] fArr4 : fArr3) {
            if (fArr4.length != 2 || fArr4[0] >= fArr4[1]) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        if (this.schemaVersion == 2) {
            float[][] fArr5 = this.formantToMode;
            if (fArr5.length != this.areaModes.length) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            for (float[] fArr6 : fArr5) {
                if (fArr6.length != this.referenceFormantsHz.length) {
                    throw new IllegalArgumentException("Failed requirement.".toString());
                }
            }
            if (this.modeLabels.size() != this.areaModes.length) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
    }

    public final int getSchemaVersion() {
        return this.schemaVersion;
    }

    public final String getModelId() {
        return this.modelId;
    }

    public final int getSampleRateHz() {
        return this.sampleRateHz;
    }

    public final int getFrameSize() {
        return this.frameSize;
    }

    public final int getHopSize() {
        return this.hopSize;
    }

    public final float[] getSectionPosition() {
        return this.sectionPosition;
    }

    public final float[] getAreaMeanCm2() {
        return this.areaMeanCm2;
    }

    public final float[][] getAreaModes() {
        return this.areaModes;
    }

    public final float[] getReferenceFormantsHz() {
        return this.referenceFormantsHz;
    }

    public final float[] getCoefficientScaleHz() {
        return this.coefficientScaleHz;
    }

    /* JADX WARN: Illegal instructions before constructor call */
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    public /* synthetic */ ReducedModelAsset(int i, String str, int i2, int i3, int i4, float[] fArr, float[] fArr2, float[][] fArr3, float[] fArr4, float[] fArr5, float[][] fArr6, float[][] fArr7, List list, String str2, String str3, int i5, int i6, boolean z, float f, int i7, DefaultConstructorMarker defaultConstructorMarker) {
        this(i, str, i2, i3, r5, fArr, fArr2, fArr3, fArr4, fArr5, r11, r12, r13, (i7 & ConstantsKt.DEFAULT_BUFFER_SIZE) != 0 ? "legacy" : str2, (i7 & 16384) != 0 ? "legacy" : str3, (32768 & i7) != 0 ? 0 : i5, (65536 & i7) != 0 ? 0 : i6, (131072 & i7) != 0 ? false : z, (i7 & 262144) != 0 ? 0.22f : f);
        float[][] fArr8;
        List list2;
        int i8 = (i7 & 16) != 0 ? i3 : i4;
        float[][] fArr9 = (i7 & 1024) != 0 ? new float[0][] : fArr6;
        if ((i7 & 2048) != 0) {
            int length = fArr3.length;
            float[][] fArr10 = new float[length][];
            for (int i9 = 0; i9 < length; i9++) {
                fArr10[i9] = new float[]{-2.0f, 2.0f};
            }
            fArr8 = fArr10;
        } else {
            fArr8 = fArr7;
        }
        if ((i7 & ConstantsKt.DEFAULT_BLOCK_SIZE) != 0) {
            int length2 = fArr3.length;
            ArrayList arrayList = new ArrayList(length2);
            int i10 = 0;
            while (i10 < length2) {
                i10++;
                arrayList.add("Reduced mode " + i10);
            }
            list2 = arrayList;
        } else {
            list2 = list;
        }
    }

    public final float[][] getFormantToMode() {
        return this.formantToMode;
    }

    public final float[][] getCoefficientLimitsSd() {
        return this.coefficientLimitsSd;
    }

    public final List<String> getModeLabels() {
        return this.modeLabels;
    }

    public final String getAtlasSourceModelSha256() {
        return this.atlasSourceModelSha256;
    }

    public final String getAtlasFreezeManifestSha256() {
        return this.atlasFreezeManifestSha256;
    }

    public final int getAtlasSubjects() {
        return this.atlasSubjects;
    }

    public final int getIndependentExpertAcceptances() {
        return this.independentExpertAcceptances;
    }

    public final boolean getScientificReleaseReady() {
        return this.scientificReleaseReady;
    }

    public final float getAbstentionConfidenceThreshold() {
        return this.abstentionConfidenceThreshold;
    }

    public final Pair<float[], float[]> inferArea(List<Float> formantsHz) {
        Intrinsics.checkNotNullParameter(formantsHz, "formantsHz");
        int length = this.areaModes.length;
        float[] fArr = new float[length];
        int min = Math.min(formantsHz.size(), this.referenceFormantsHz.length);
        if (this.schemaVersion == 2) {
            for (int i = 0; i < length; i++) {
                float f = 0.0f;
                for (int i2 = 0; i2 < min; i2++) {
                    f += this.formantToMode[i][i2] * (formantsHz.get(i2).floatValue() - this.referenceFormantsHz[i2]);
                }
                float[] fArr2 = this.coefficientLimitsSd[i];
                fArr[i] = RangesKt.coerceIn(f, fArr2[0], fArr2[1]);
            }
        } else {
            for (int i3 = 0; i3 < min; i3++) {
                float floatValue = (formantsHz.get(i3).floatValue() - this.referenceFormantsHz[i3]) / this.coefficientScaleHz[i3];
                float[] fArr3 = this.coefficientLimitsSd[i3];
                fArr[i3] = RangesKt.coerceIn(floatValue, fArr3[0], fArr3[1]);
            }
        }
        return new Pair<>(areaFromCoefficients(fArr), fArr);
    }

    public final float[] areaFromCoefficients(float[] input) {
        Intrinsics.checkNotNullParameter(input, "input");
        if (input.length != this.areaModes.length) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        int length = this.areaMeanCm2.length;
        float[] fArr = new float[length];
        for (int i = 0; i < length; i++) {
            float log = (float) Math.log(this.areaMeanCm2[i]);
            int length2 = this.areaModes.length;
            for (int i2 = 0; i2 < length2; i2++) {
                float[] fArr2 = this.coefficientLimitsSd[i2];
                log += RangesKt.coerceIn(input[i2], fArr2[0], fArr2[1]) * this.areaModes[i2][i];
            }
            float exp = (float) Math.exp(log);
            float f = this.areaMeanCm2[i];
            fArr[i] = RangesKt.coerceIn(exp, 0.2f * f, f * 5.0f);
        }
        return fArr;
    }

    /* compiled from: ReducedModelAsset.kt */
    @Metadata(d1 = {"\u0000 \n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\b\u0086\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u000e\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u0007J\u0015\u0010\b\u001a\u00020\u00052\u0006\u0010\t\u001a\u00020\nH\u0000¢\u0006\u0002\b\u000b"}, d2 = {"Lorg/vocaltract/pixel/ReducedModelAsset$Companion;", "", "<init>", "()V", "load", "Lorg/vocaltract/pixel/ReducedModelAsset;", "context", "Landroid/content/Context;", "parse", "json", "Lorg/json/JSONObject;", "parse$main"}, k = 1, mv = {2, 0, 0}, xi = 48)
    public static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }

        /* JADX DEBUG: Finally have unexpected throw blocks count: 2, expect 1 */
        public final ReducedModelAsset load(Context context) {
            Intrinsics.checkNotNullParameter(context, "context");
            InputStream openRawResource = context.getResources().openRawResource(R.raw.reduced_model);
            Intrinsics.checkNotNullExpressionValue(openRawResource, "openRawResource(...)");
            Reader inputStreamReader = new InputStreamReader(openRawResource, Charsets.UTF_8);
            BufferedReader bufferedReader = inputStreamReader instanceof BufferedReader ? (BufferedReader) inputStreamReader : new BufferedReader(inputStreamReader, ConstantsKt.DEFAULT_BUFFER_SIZE);
            try {
                String readText = TextStreamsKt.readText(bufferedReader);
                CloseableKt.closeFinally(bufferedReader, null);
                return parse$main(new JSONObject(readText));
            } finally {
            }
        }

        /* JADX WARN: Code restructure failed: missing block: B:14:0x0071, code lost:
        
            r5 = org.vocaltract.pixel.ReducedModelAssetKt.toStringList(r5);
         */
        /* JADX WARN: Code restructure failed: missing block: B:19:0x00ef, code lost:
        
            r0 = org.vocaltract.pixel.ReducedModelAssetKt.toFloatArray2d(r0);
         */
        /* JADX WARN: Code restructure failed: missing block: B:9:0x004b, code lost:
        
            r5 = org.vocaltract.pixel.ReducedModelAssetKt.toFloatArray2d(r5);
         */
        /* JADX WARN: Multi-variable type inference failed */
        /* JADX WARN: Type inference failed for: r5v13, types: [java.util.List] */
        /*
            Code decompiled incorrectly, please refer to instructions dump.
        */
        public final ReducedModelAsset parse$main(JSONObject json) {
            float[][] floatArray2d;
            float[] floatArray;
            float[][] fArr;
            ArrayList arrayList;
            float[] floatArray2;
            float[] floatArray3;
            float[] floatArray4;
            float[][] fArr2;
            ?? stringList;
            float[][] floatArray2d2;
            Intrinsics.checkNotNullParameter(json, "json");
            int i = json.getInt("schema_version");
            JSONArray jSONArray = json.getJSONArray("area_modes");
            Intrinsics.checkNotNullExpressionValue(jSONArray, "getJSONArray(...)");
            floatArray2d = ReducedModelAssetKt.toFloatArray2d(jSONArray);
            JSONArray jSONArray2 = json.getJSONArray("reference_formants_hz");
            Intrinsics.checkNotNullExpressionValue(jSONArray2, "getJSONArray(...)");
            floatArray = ReducedModelAssetKt.toFloatArray(jSONArray2);
            JSONObject optJSONObject = json.optJSONObject("atlas");
            if (optJSONObject == null) {
                optJSONObject = new JSONObject();
            }
            JSONObject optJSONObject2 = json.optJSONObject("uncertainty");
            if (optJSONObject2 == null) {
                optJSONObject2 = new JSONObject();
            }
            JSONArray optJSONArray = json.optJSONArray("coefficient_limits_sd");
            if (optJSONArray == null || floatArray2d2 == null) {
                int length = floatArray2d.length;
                float[][] fArr3 = new float[length][];
                for (int i2 = 0; i2 < length; i2++) {
                    fArr3[i2] = new float[]{-2.0f, 2.0f};
                }
                fArr = fArr3;
            } else {
                fArr = floatArray2d2;
            }
            JSONArray optJSONArray2 = json.optJSONArray("mode_labels");
            if (optJSONArray2 == null || stringList == 0) {
                int length2 = floatArray2d.length;
                ArrayList arrayList2 = new ArrayList(length2);
                int i3 = 0;
                while (i3 < length2) {
                    i3++;
                    arrayList2.add("Reduced mode " + i3);
                }
                arrayList = arrayList2;
            } else {
                arrayList = stringList;
            }
            String string = json.getString("model_id");
            Intrinsics.checkNotNullExpressionValue(string, "getString(...)");
            int i4 = json.getInt("sample_rate_hz");
            int i5 = json.getInt("frame_size");
            int optInt = json.optInt("hop_size", json.getInt("frame_size"));
            JSONArray jSONArray3 = json.getJSONArray("section_position");
            Intrinsics.checkNotNullExpressionValue(jSONArray3, "getJSONArray(...)");
            floatArray2 = ReducedModelAssetKt.toFloatArray(jSONArray3);
            JSONArray jSONArray4 = json.getJSONArray("area_mean_cm2");
            Intrinsics.checkNotNullExpressionValue(jSONArray4, "getJSONArray(...)");
            floatArray3 = ReducedModelAssetKt.toFloatArray(jSONArray4);
            JSONArray jSONArray5 = json.getJSONArray("coefficient_scale_hz");
            Intrinsics.checkNotNullExpressionValue(jSONArray5, "getJSONArray(...)");
            floatArray4 = ReducedModelAssetKt.toFloatArray(jSONArray5);
            JSONArray optJSONArray3 = json.optJSONArray("formant_to_mode");
            if (optJSONArray3 == null || fArr2 == null) {
                fArr2 = new float[0][];
            }
            String optString = optJSONObject.optString("source_model_sha256", "legacy");
            Intrinsics.checkNotNullExpressionValue(optString, "optString(...)");
            String optString2 = optJSONObject.optString("freeze_manifest_sha256", "legacy");
            Intrinsics.checkNotNullExpressionValue(optString2, "optString(...)");
            return new ReducedModelAsset(i, string, i4, i5, optInt, floatArray2, floatArray3, floatArray2d, floatArray, floatArray4, fArr2, fArr, arrayList, optString, optString2, optJSONObject.optInt("subjects", 0), optJSONObject.optInt("independent_expert_acceptances", 0), optJSONObject.optBoolean("scientific_release_ready", false), (float) optJSONObject2.optDouble("abstention_confidence_threshold", 0.22d));
        }
    }
}

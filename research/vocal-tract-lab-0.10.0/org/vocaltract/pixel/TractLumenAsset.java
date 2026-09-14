package org.vocaltract.pixel;

import android.content.Context;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.Arrays;
import java.util.Collection;
import java.util.Iterator;
import java.util.List;
import java.util.zip.CRC32;
import kotlin.Lazy;
import kotlin.LazyKt;
import kotlin.LazyThreadSafetyMode;
import kotlin.Metadata;
import kotlin.collections.ArraysKt;
import kotlin.collections.CollectionsKt;
import kotlin.collections.IntIterator;
import kotlin.io.ByteStreamsKt;
import kotlin.io.CloseableKt;
import kotlin.jvm.functions.Function0;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: TractLumenAsset.kt */
@Metadata(d1 = {"\u0000>\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\b\n\u0002\b\u0003\n\u0002\u0010\u0014\n\u0002\b\u0004\n\u0002\u0010\u0015\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0003\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010 \n\u0002\b?\n\u0002\u0010\u000b\n\u0002\b\u0004\b\u0086\b\u0018\u0000 X2\u00020\u0001:\u0001XBu\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0003\u0012\u0006\u0010\u0006\u001a\u00020\u0007\u0012\u0006\u0010\b\u001a\u00020\u0007\u0012\u0006\u0010\t\u001a\u00020\u0007\u0012\u0006\u0010\n\u001a\u00020\u0007\u0012\u0006\u0010\u000b\u001a\u00020\f\u0012\u0006\u0010\r\u001a\u00020\u000e\u0012\u0006\u0010\u000f\u001a\u00020\u000e\u0012\u0006\u0010\u0010\u001a\u00020\u0003\u0012\u0006\u0010\u0011\u001a\u00020\u0012\u0012\f\u0010\u0013\u001a\b\u0012\u0004\u0012\u00020\u00120\u0014¢\u0006\u0004\b\u0015\u0010\u0016J\u000e\u0010A\u001a\u00020\u00072\u0006\u0010B\u001a\u00020\u0007J\u0010\u0010C\u001a\u00020\u000e2\u0006\u0010D\u001a\u00020\u0003H\u0002J\t\u0010E\u001a\u00020\u0003HÆ\u0003J\t\u0010F\u001a\u00020\u0003HÆ\u0003J\t\u0010G\u001a\u00020\u0003HÆ\u0003J\t\u0010H\u001a\u00020\u0007HÆ\u0003J\t\u0010I\u001a\u00020\u0007HÆ\u0003J\t\u0010J\u001a\u00020\u0007HÆ\u0003J\t\u0010K\u001a\u00020\u0007HÆ\u0003J\t\u0010L\u001a\u00020\fHÆ\u0003J\t\u0010M\u001a\u00020\u000eHÆ\u0003J\t\u0010N\u001a\u00020\u000eHÆ\u0003J\t\u0010O\u001a\u00020\u0003HÆ\u0003J\t\u0010P\u001a\u00020\u0012HÆ\u0003J\u000f\u0010Q\u001a\b\u0012\u0004\u0012\u00020\u00120\u0014HÆ\u0003J\u0091\u0001\u0010R\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00032\b\b\u0002\u0010\u0005\u001a\u00020\u00032\b\b\u0002\u0010\u0006\u001a\u00020\u00072\b\b\u0002\u0010\b\u001a\u00020\u00072\b\b\u0002\u0010\t\u001a\u00020\u00072\b\b\u0002\u0010\n\u001a\u00020\u00072\b\b\u0002\u0010\u000b\u001a\u00020\f2\b\b\u0002\u0010\r\u001a\u00020\u000e2\b\b\u0002\u0010\u000f\u001a\u00020\u000e2\b\b\u0002\u0010\u0010\u001a\u00020\u00032\b\b\u0002\u0010\u0011\u001a\u00020\u00122\u000e\b\u0002\u0010\u0013\u001a\b\u0012\u0004\u0012\u00020\u00120\u0014HÆ\u0001J\u0013\u0010S\u001a\u00020T2\b\u0010U\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010V\u001a\u00020\u0003HÖ\u0001J\t\u0010W\u001a\u00020\u0012HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0017\u0010\u0018R\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0019\u0010\u0018R\u0011\u0010\u0005\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u001a\u0010\u0018R\u0011\u0010\u0006\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b\u001b\u0010\u001cR\u0011\u0010\b\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b\u001d\u0010\u001cR\u0011\u0010\t\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b\u001e\u0010\u001cR\u0011\u0010\n\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b\u001f\u0010\u001cR\u0011\u0010\u000b\u001a\u00020\f¢\u0006\b\n\u0000\u001a\u0004\b \u0010!R\u0011\u0010\r\u001a\u00020\u000e¢\u0006\b\n\u0000\u001a\u0004\b\"\u0010#R\u0011\u0010\u000f\u001a\u00020\u000e¢\u0006\b\n\u0000\u001a\u0004\b$\u0010#R\u0011\u0010\u0010\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b%\u0010\u0018R\u0011\u0010\u0011\u001a\u00020\u0012¢\u0006\b\n\u0000\u001a\u0004\b&\u0010'R\u0017\u0010\u0013\u001a\b\u0012\u0004\u0012\u00020\u00120\u0014¢\u0006\b\n\u0000\u001a\u0004\b(\u0010)R\u0011\u0010*\u001a\u00020\u0012¢\u0006\b\n\u0000\u001a\u0004\b+\u0010'R\u0011\u0010,\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b-\u0010\u0018R\u0011\u0010.\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b/\u0010\u0018R\u0011\u00100\u001a\u00020\u00038F¢\u0006\u0006\u001a\u0004\b1\u0010\u0018R\u0011\u00102\u001a\u00020\u00038F¢\u0006\u0006\u001a\u0004\b3\u0010\u0018R\u0011\u00104\u001a\u00020\u00078F¢\u0006\u0006\u001a\u0004\b5\u0010\u001cR\u0011\u00106\u001a\u00020\f8F¢\u0006\u0006\u001a\u0004\b7\u0010!R\u0011\u00108\u001a\u00020\u00078F¢\u0006\u0006\u001a\u0004\b9\u0010\u001cR\u001b\u0010:\u001a\u00020\f8FX\u0086\u0084\u0002¢\u0006\f\n\u0004\b<\u0010=\u001a\u0004\b;\u0010!R\u001b\u0010>\u001a\u00020\u00078FX\u0086\u0084\u0002¢\u0006\f\n\u0004\b@\u0010=\u001a\u0004\b?\u0010\u001c"}, d2 = {"Lorg/vocaltract/pixel/TractLumenAsset;", "", "schemaVersion", "", "sectionCount", "angularSamples", "sectionPosition", "", "referenceAreaCm2", "centerlineMm", "ringOffsetsMm", "triangleIndices", "", "minimumAreaRatio", "", "maximumAreaRatio", "provenanceKind", "provenance", "", "sourceSha256", "", "<init>", "(III[F[F[F[F[IFFILjava/lang/String;Ljava/util/List;)V", "getSchemaVersion", "()I", "getSectionCount", "getAngularSamples", "getSectionPosition", "()[F", "getReferenceAreaCm2", "getCenterlineMm", "getRingOffsetsMm", "getTriangleIndices", "()[I", "getMinimumAreaRatio", "()F", "getMaximumAreaRatio", "getProvenanceKind", "getProvenance", "()Ljava/lang/String;", "getSourceSha256", "()Ljava/util/List;", "modelId", "getModelId", "vertexCount", "getVertexCount", "triangleCount", "getTriangleCount", "sections", "getSections", "rings", "getRings", "baseOffsetsMm", "getBaseOffsetsMm", "faces", "getFaces", "referenceAreasCm2", "getReferenceAreasCm2", "sectionForVertex", "getSectionForVertex", "sectionForVertex$delegate", "Lkotlin/Lazy;", "meanVertices", "getMeanVertices", "meanVertices$delegate", "morph", "areaCm2", "ringAreaCm2", "section", "component1", "component2", "component3", "component4", "component5", "component6", "component7", "component8", "component9", "component10", "component11", "component12", "component13", "copy", "equals", "", "other", "hashCode", "toString", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class TractLumenAsset {
    private static final int HEADER_BYTES = 104;
    private static final int SCHEMA_VERSION = 2;
    private final int angularSamples;
    private final float[] centerlineMm;
    private final float maximumAreaRatio;

    /* renamed from: meanVertices$delegate, reason: from kotlin metadata */
    private final Lazy meanVertices;
    private final float minimumAreaRatio;
    private final String modelId;
    private final String provenance;
    private final int provenanceKind;
    private final float[] referenceAreaCm2;
    private final float[] ringOffsetsMm;
    private final int schemaVersion;
    private final int sectionCount;

    /* renamed from: sectionForVertex$delegate, reason: from kotlin metadata */
    private final Lazy sectionForVertex;
    private final float[] sectionPosition;
    private final List<String> sourceSha256;
    private final int triangleCount;
    private final int[] triangleIndices;
    private final int vertexCount;

    /* renamed from: Companion, reason: from kotlin metadata */
    public static final Companion INSTANCE = new Companion(null);
    private static final byte[] MAGIC = {86, 84, 76, 85, 77, 78, 50, 0};

    /* renamed from: component1, reason: from getter */
    public final int getSchemaVersion() {
        return this.schemaVersion;
    }

    /* renamed from: component10, reason: from getter */
    public final float getMaximumAreaRatio() {
        return this.maximumAreaRatio;
    }

    /* renamed from: component11, reason: from getter */
    public final int getProvenanceKind() {
        return this.provenanceKind;
    }

    /* renamed from: component12, reason: from getter */
    public final String getProvenance() {
        return this.provenance;
    }

    public final List<String> component13() {
        return this.sourceSha256;
    }

    /* renamed from: component2, reason: from getter */
    public final int getSectionCount() {
        return this.sectionCount;
    }

    /* renamed from: component3, reason: from getter */
    public final int getAngularSamples() {
        return this.angularSamples;
    }

    /* renamed from: component4, reason: from getter */
    public final float[] getSectionPosition() {
        return this.sectionPosition;
    }

    /* renamed from: component5, reason: from getter */
    public final float[] getReferenceAreaCm2() {
        return this.referenceAreaCm2;
    }

    /* renamed from: component6, reason: from getter */
    public final float[] getCenterlineMm() {
        return this.centerlineMm;
    }

    /* renamed from: component7, reason: from getter */
    public final float[] getRingOffsetsMm() {
        return this.ringOffsetsMm;
    }

    /* renamed from: component8, reason: from getter */
    public final int[] getTriangleIndices() {
        return this.triangleIndices;
    }

    /* renamed from: component9, reason: from getter */
    public final float getMinimumAreaRatio() {
        return this.minimumAreaRatio;
    }

    public final TractLumenAsset copy(int schemaVersion, int sectionCount, int angularSamples, float[] sectionPosition, float[] referenceAreaCm2, float[] centerlineMm, float[] ringOffsetsMm, int[] triangleIndices, float minimumAreaRatio, float maximumAreaRatio, int provenanceKind, String provenance, List<String> sourceSha256) {
        Intrinsics.checkNotNullParameter(sectionPosition, "sectionPosition");
        Intrinsics.checkNotNullParameter(referenceAreaCm2, "referenceAreaCm2");
        Intrinsics.checkNotNullParameter(centerlineMm, "centerlineMm");
        Intrinsics.checkNotNullParameter(ringOffsetsMm, "ringOffsetsMm");
        Intrinsics.checkNotNullParameter(triangleIndices, "triangleIndices");
        Intrinsics.checkNotNullParameter(provenance, "provenance");
        Intrinsics.checkNotNullParameter(sourceSha256, "sourceSha256");
        return new TractLumenAsset(schemaVersion, sectionCount, angularSamples, sectionPosition, referenceAreaCm2, centerlineMm, ringOffsetsMm, triangleIndices, minimumAreaRatio, maximumAreaRatio, provenanceKind, provenance, sourceSha256);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof TractLumenAsset)) {
            return false;
        }
        TractLumenAsset tractLumenAsset = (TractLumenAsset) other;
        return this.schemaVersion == tractLumenAsset.schemaVersion && this.sectionCount == tractLumenAsset.sectionCount && this.angularSamples == tractLumenAsset.angularSamples && Intrinsics.areEqual(this.sectionPosition, tractLumenAsset.sectionPosition) && Intrinsics.areEqual(this.referenceAreaCm2, tractLumenAsset.referenceAreaCm2) && Intrinsics.areEqual(this.centerlineMm, tractLumenAsset.centerlineMm) && Intrinsics.areEqual(this.ringOffsetsMm, tractLumenAsset.ringOffsetsMm) && Intrinsics.areEqual(this.triangleIndices, tractLumenAsset.triangleIndices) && Float.compare(this.minimumAreaRatio, tractLumenAsset.minimumAreaRatio) == 0 && Float.compare(this.maximumAreaRatio, tractLumenAsset.maximumAreaRatio) == 0 && this.provenanceKind == tractLumenAsset.provenanceKind && Intrinsics.areEqual(this.provenance, tractLumenAsset.provenance) && Intrinsics.areEqual(this.sourceSha256, tractLumenAsset.sourceSha256);
    }

    public int hashCode() {
        return (((((((((((((((((((((((Integer.hashCode(this.schemaVersion) * 31) + Integer.hashCode(this.sectionCount)) * 31) + Integer.hashCode(this.angularSamples)) * 31) + Arrays.hashCode(this.sectionPosition)) * 31) + Arrays.hashCode(this.referenceAreaCm2)) * 31) + Arrays.hashCode(this.centerlineMm)) * 31) + Arrays.hashCode(this.ringOffsetsMm)) * 31) + Arrays.hashCode(this.triangleIndices)) * 31) + Float.hashCode(this.minimumAreaRatio)) * 31) + Float.hashCode(this.maximumAreaRatio)) * 31) + Integer.hashCode(this.provenanceKind)) * 31) + this.provenance.hashCode()) * 31) + this.sourceSha256.hashCode();
    }

    public String toString() {
        return "TractLumenAsset(schemaVersion=" + this.schemaVersion + ", sectionCount=" + this.sectionCount + ", angularSamples=" + this.angularSamples + ", sectionPosition=" + Arrays.toString(this.sectionPosition) + ", referenceAreaCm2=" + Arrays.toString(this.referenceAreaCm2) + ", centerlineMm=" + Arrays.toString(this.centerlineMm) + ", ringOffsetsMm=" + Arrays.toString(this.ringOffsetsMm) + ", triangleIndices=" + Arrays.toString(this.triangleIndices) + ", minimumAreaRatio=" + this.minimumAreaRatio + ", maximumAreaRatio=" + this.maximumAreaRatio + ", provenanceKind=" + this.provenanceKind + ", provenance=" + this.provenance + ", sourceSha256=" + this.sourceSha256 + ")";
    }

    public TractLumenAsset(int i, int i2, int i3, float[] sectionPosition, float[] referenceAreaCm2, float[] centerlineMm, float[] ringOffsetsMm, int[] triangleIndices, float f, float f2, int i4, String provenance, List<String> sourceSha256) {
        String str;
        Intrinsics.checkNotNullParameter(sectionPosition, "sectionPosition");
        Intrinsics.checkNotNullParameter(referenceAreaCm2, "referenceAreaCm2");
        Intrinsics.checkNotNullParameter(centerlineMm, "centerlineMm");
        Intrinsics.checkNotNullParameter(ringOffsetsMm, "ringOffsetsMm");
        Intrinsics.checkNotNullParameter(triangleIndices, "triangleIndices");
        Intrinsics.checkNotNullParameter(provenance, "provenance");
        Intrinsics.checkNotNullParameter(sourceSha256, "sourceSha256");
        this.schemaVersion = i;
        this.sectionCount = i2;
        this.angularSamples = i3;
        this.sectionPosition = sectionPosition;
        this.referenceAreaCm2 = referenceAreaCm2;
        this.centerlineMm = centerlineMm;
        this.ringOffsetsMm = ringOffsetsMm;
        this.triangleIndices = triangleIndices;
        this.minimumAreaRatio = f;
        this.maximumAreaRatio = f2;
        this.provenanceKind = i4;
        this.provenance = provenance;
        this.sourceSha256 = sourceSha256;
        if (i4 == 2) {
            str = "vt3d-metric-mri-engineering-mean-v0.6.3";
        } else {
            str = "vt3d-sex-informed-a-blend-v2";
        }
        this.modelId = str;
        this.vertexCount = (i2 * i3) + 2;
        this.triangleCount = triangleIndices.length / 3;
        this.sectionForVertex = LazyKt.lazy(LazyThreadSafetyMode.NONE, new Function0() { // from class: org.vocaltract.pixel.TractLumenAsset$$ExternalSyntheticLambda0
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                int[] sectionForVertex_delegate$lambda$0;
                sectionForVertex_delegate$lambda$0 = TractLumenAsset.sectionForVertex_delegate$lambda$0(TractLumenAsset.this);
                return sectionForVertex_delegate$lambda$0;
            }
        });
        this.meanVertices = LazyKt.lazy(LazyThreadSafetyMode.NONE, new Function0() { // from class: org.vocaltract.pixel.TractLumenAsset$$ExternalSyntheticLambda1
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                float[] meanVertices_delegate$lambda$1;
                meanVertices_delegate$lambda$1 = TractLumenAsset.meanVertices_delegate$lambda$1(TractLumenAsset.this);
                return meanVertices_delegate$lambda$1;
            }
        });
        if (i != 2) {
            throw new IllegalArgumentException(("Unsupported lumen schema " + i).toString());
        }
        if (8 > i2 || i2 >= 257) {
            throw new IllegalArgumentException(("Invalid section count " + i2).toString());
        }
        if (8 > i3 || i3 >= 257 || i3 % 2 != 0) {
            throw new IllegalArgumentException(("Invalid angular sample count " + i3).toString());
        }
        if (sectionPosition.length != i2) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (referenceAreaCm2.length != i2) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (centerlineMm.length != i2 * 3) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (ringOffsetsMm.length != i2 * i3 * 3) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (triangleIndices.length != i2 * 6 * i3) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (Float.isInfinite(f) || Float.isNaN(f) || Float.isInfinite(f2) || Float.isNaN(f2)) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (f <= 0.0f || f2 < f) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        for (float f3 : sectionPosition) {
            if (Float.isInfinite(f3) || Float.isNaN(f3)) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        for (float f4 : this.referenceAreaCm2) {
            if (Float.isInfinite(f4) || Float.isNaN(f4) || f4 <= 0.0f) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        for (float f5 : this.centerlineMm) {
            if (Float.isInfinite(f5) || Float.isNaN(f5)) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        for (float f6 : this.ringOffsetsMm) {
            if (Float.isInfinite(f6) || Float.isNaN(f6)) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        for (int i5 : this.triangleIndices) {
            if (i5 < 0 || i5 >= this.vertexCount) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        if (Math.abs(ArraysKt.first(this.sectionPosition)) >= 1.0E-5f) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (Math.abs(ArraysKt.last(this.sectionPosition) - 1.0f) >= 1.0E-5f) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        Iterable until = RangesKt.until(1, this.sectionCount);
        if (!(until instanceof Collection) || !((Collection) until).isEmpty()) {
            Iterator it = until.iterator();
            while (it.hasNext()) {
                int nextInt = ((IntIterator) it).nextInt();
                float[] fArr = this.sectionPosition;
                if (fArr[nextInt] <= fArr[nextInt - 1]) {
                    throw new IllegalArgumentException("Failed requirement.".toString());
                }
            }
        }
        int i6 = this.sectionCount;
        for (int i7 = 0; i7 < i6; i7++) {
            float ringAreaCm2 = ringAreaCm2(i7);
            float abs = Math.abs(ringAreaCm2 - this.referenceAreaCm2[i7]);
            float f7 = this.referenceAreaCm2[i7];
            float f8 = abs / f7;
            if (ringAreaCm2 <= 0.0f || f8 >= 0.002f) {
                throw new IllegalArgumentException(("Reference area mismatch at section " + i7 + ": " + ringAreaCm2 + " vs " + f7).toString());
            }
        }
    }

    public final int getSchemaVersion() {
        return this.schemaVersion;
    }

    public final int getSectionCount() {
        return this.sectionCount;
    }

    public final int getAngularSamples() {
        return this.angularSamples;
    }

    public final float[] getSectionPosition() {
        return this.sectionPosition;
    }

    public final float[] getReferenceAreaCm2() {
        return this.referenceAreaCm2;
    }

    public final float[] getCenterlineMm() {
        return this.centerlineMm;
    }

    public final float[] getRingOffsetsMm() {
        return this.ringOffsetsMm;
    }

    public final int[] getTriangleIndices() {
        return this.triangleIndices;
    }

    public final float getMinimumAreaRatio() {
        return this.minimumAreaRatio;
    }

    public final float getMaximumAreaRatio() {
        return this.maximumAreaRatio;
    }

    public final int getProvenanceKind() {
        return this.provenanceKind;
    }

    public final String getProvenance() {
        return this.provenance;
    }

    public final List<String> getSourceSha256() {
        return this.sourceSha256;
    }

    public final String getModelId() {
        return this.modelId;
    }

    public final int getVertexCount() {
        return this.vertexCount;
    }

    public final int getTriangleCount() {
        return this.triangleCount;
    }

    public final int getSections() {
        return this.sectionCount;
    }

    public final int getRings() {
        return this.angularSamples;
    }

    public final float[] getBaseOffsetsMm() {
        return this.ringOffsetsMm;
    }

    public final int[] getFaces() {
        return this.triangleIndices;
    }

    public final float[] getReferenceAreasCm2() {
        return this.referenceAreaCm2;
    }

    public final int[] getSectionForVertex() {
        return (int[]) this.sectionForVertex.getValue();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final int[] sectionForVertex_delegate$lambda$0(TractLumenAsset tractLumenAsset) {
        int i;
        int i2 = tractLumenAsset.vertexCount;
        int[] iArr = new int[i2];
        for (int i3 = 0; i3 < i2; i3++) {
            int i4 = tractLumenAsset.vertexCount;
            if (i3 == i4 - 2) {
                i = 0;
            } else if (i3 == i4 - 1) {
                i = tractLumenAsset.sectionCount - 1;
            } else {
                i = i3 / tractLumenAsset.angularSamples;
            }
            iArr[i3] = i;
        }
        return iArr;
    }

    public final float[] getMeanVertices() {
        return (float[]) this.meanVertices.getValue();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final float[] meanVertices_delegate$lambda$1(TractLumenAsset tractLumenAsset) {
        return tractLumenAsset.morph(tractLumenAsset.referenceAreaCm2);
    }

    public final float[] morph(float[] areaCm2) {
        Intrinsics.checkNotNullParameter(areaCm2, "areaCm2");
        int length = areaCm2.length;
        int i = this.sectionCount;
        if (length != i) {
            throw new IllegalArgumentException(("Expected " + i + " area samples, received " + areaCm2.length).toString());
        }
        for (float f : areaCm2) {
            if (Float.isInfinite(f) || Float.isNaN(f) || f <= 0.0f) {
                throw new IllegalArgumentException("Area samples must be finite and positive".toString());
            }
        }
        float[] fArr = new float[this.vertexCount * 3];
        int i2 = this.sectionCount;
        for (int i3 = 0; i3 < i2; i3++) {
            float sqrt = (float) Math.sqrt(RangesKt.coerceIn(areaCm2[i3] / this.referenceAreaCm2[i3], this.minimumAreaRatio, this.maximumAreaRatio));
            int i4 = i3 * 3;
            int i5 = this.angularSamples;
            for (int i6 = 0; i6 < i5; i6++) {
                int i7 = ((this.angularSamples * i3) + i6) * 3;
                float[] fArr2 = this.centerlineMm;
                float f2 = fArr2[i4];
                float[] fArr3 = this.ringOffsetsMm;
                fArr[i7] = f2 + (fArr3[i7] * sqrt);
                int i8 = i7 + 1;
                fArr[i8] = fArr2[i4 + 1] + (fArr3[i8] * sqrt);
                int i9 = i7 + 2;
                fArr[i9] = fArr2[i4 + 2] + (fArr3[i9] * sqrt);
            }
        }
        int i10 = this.vertexCount;
        int i11 = (i10 - 2) * 3;
        float[] fArr4 = this.centerlineMm;
        fArr[i11] = fArr4[0];
        fArr[i11 + 1] = fArr4[1];
        fArr[i11 + 2] = fArr4[2];
        int i12 = (i10 - 1) * 3;
        int i13 = (this.sectionCount - 1) * 3;
        fArr[i12] = fArr4[i13];
        fArr[i12 + 1] = fArr4[i13 + 1];
        fArr[i12 + 2] = fArr4[i13 + 2];
        return fArr;
    }

    private final float ringAreaCm2(int section) {
        TractLumenAsset tractLumenAsset = this;
        int i = tractLumenAsset.angularSamples;
        int i2 = section * i * 3;
        double d = 0.0d;
        double d2 = 0.0d;
        int i3 = 0;
        double d3 = 0.0d;
        while (i3 < i) {
            int i4 = (i3 * 3) + i2;
            int i5 = i3 + 1;
            int i6 = ((i5 % tractLumenAsset.angularSamples) * 3) + i2;
            float[] fArr = tractLumenAsset.ringOffsetsMm;
            double d4 = fArr[i4];
            double d5 = fArr[i4 + 1];
            double d6 = fArr[i4 + 2];
            double d7 = d2;
            double d8 = fArr[i6];
            double d9 = d3;
            double d10 = fArr[i6 + 1];
            double d11 = fArr[i6 + 2];
            d += (d5 * d11) - (d6 * d10);
            double d12 = d9 + ((d6 * d8) - (d11 * d4));
            d2 = d7 + ((d4 * d10) - (d5 * d8));
            tractLumenAsset = this;
            d3 = d12;
            i = i;
            i3 = i5;
        }
        double d13 = d3;
        double d14 = d2;
        return (float) ((Math.sqrt(((d * d) + (d13 * d13)) + (d14 * d14)) * 0.5d) / 100.0d);
    }

    /* compiled from: TractLumenAsset.kt */
    @Metadata(d1 = {"\u00000\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\b\n\u0002\b\u0002\n\u0002\u0010\u0012\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0002\b\u0002\b\u0086\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u000e\u0010\t\u001a\u00020\n2\u0006\u0010\u000b\u001a\u00020\fJ\u0015\u0010\r\u001a\u00020\n2\u0006\u0010\u000e\u001a\u00020\u000fH\u0000¢\u0006\u0002\b\u0010J\u0015\u0010\r\u001a\u00020\n2\u0006\u0010\u0011\u001a\u00020\bH\u0000¢\u0006\u0002\b\u0010R\u000e\u0010\u0004\u001a\u00020\u0005X\u0082T¢\u0006\u0002\n\u0000R\u000e\u0010\u0006\u001a\u00020\u0005X\u0082T¢\u0006\u0002\n\u0000R\u000e\u0010\u0007\u001a\u00020\bX\u0082\u0004¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/TractLumenAsset$Companion;", "", "<init>", "()V", "SCHEMA_VERSION", "", "HEADER_BYTES", "MAGIC", "", "load", "Lorg/vocaltract/pixel/TractLumenAsset;", "context", "Landroid/content/Context;", "parse", "input", "Ljava/io/InputStream;", "parse$main", "bytes"}, k = 1, mv = {2, 0, 0}, xi = 48)
    public static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }

        /* JADX DEBUG: Finally have unexpected throw blocks count: 2, expect 1 */
        public final TractLumenAsset load(Context context) {
            Intrinsics.checkNotNullParameter(context, "context");
            InputStream openRawResource = context.getResources().openRawResource(R.raw.tract_lumen_v2);
            Intrinsics.checkNotNullExpressionValue(openRawResource, "openRawResource(...)");
            InputStream inputStream = openRawResource;
            try {
                TractLumenAsset parse$main = parse$main(inputStream);
                CloseableKt.closeFinally(inputStream, null);
                return parse$main;
            } finally {
            }
        }

        public final TractLumenAsset parse$main(InputStream input) {
            Intrinsics.checkNotNullParameter(input, "input");
            return parse$main(ByteStreamsKt.readBytes(input));
        }

        /* JADX DEBUG: Class process forced to load method for inline: org.vocaltract.pixel.TractLumenAssetKt.access$unsignedShort(java.nio.ByteBuffer):int */
        public final TractLumenAsset parse$main(byte[] bytes) {
            int unsignedShort;
            int unsignedShort2;
            int unsignedShort3;
            int unsignedShort4;
            int unsignedShort5;
            int unsignedShort6;
            float[] readFloatArray;
            float[] readFloatArray2;
            float[] readFloatArray3;
            float[] readFloatArray4;
            String str;
            String hex;
            String hex2;
            int unsignedShort7;
            Intrinsics.checkNotNullParameter(bytes, "bytes");
            if (bytes.length < TractLumenAsset.HEADER_BYTES) {
                throw new IllegalArgumentException("Lumen asset is shorter than its header".toString());
            }
            ByteBuffer order = ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN);
            byte[] bArr = new byte[TractLumenAsset.MAGIC.length];
            order.get(bArr);
            if (!Arrays.equals(bArr, TractLumenAsset.MAGIC)) {
                throw new IllegalArgumentException("Invalid lumen asset magic".toString());
            }
            Intrinsics.checkNotNull(order);
            unsignedShort = TractLumenAssetKt.unsignedShort(order);
            unsignedShort2 = TractLumenAssetKt.unsignedShort(order);
            unsignedShort3 = TractLumenAssetKt.unsignedShort(order);
            unsignedShort4 = TractLumenAssetKt.unsignedShort(order);
            int i = order.getInt();
            int i2 = order.getInt();
            float f = order.getFloat();
            float f2 = order.getFloat();
            long j = order.getInt() & 4294967295L;
            unsignedShort5 = TractLumenAssetKt.unsignedShort(order);
            unsignedShort6 = TractLumenAssetKt.unsignedShort(order);
            byte[] bArr2 = new byte[32];
            order.get(bArr2);
            byte[] bArr3 = new byte[32];
            order.get(bArr3);
            if (unsignedShort != 2) {
                throw new IllegalArgumentException(("Unsupported lumen schema " + unsignedShort).toString());
            }
            if (unsignedShort2 != TractLumenAsset.HEADER_BYTES) {
                throw new IllegalArgumentException(("Invalid lumen header size " + unsignedShort2).toString());
            }
            if (unsignedShort6 != 0) {
                throw new IllegalArgumentException("Nonzero reserved lumen-header field".toString());
            }
            if (8 > unsignedShort3 || unsignedShort3 >= 257 || 8 > unsignedShort4 || unsignedShort4 >= 257 || unsignedShort4 % 2 != 0) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            int i3 = unsignedShort3 * unsignedShort4;
            if (i != i3 + 2) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            if (i2 != unsignedShort3 * 6 * unsignedShort4) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            if (bytes.length != unsignedShort2 + (unsignedShort3 * 4 * ((unsignedShort4 * 3) + 5)) + (i2 * 2)) {
                throw new IllegalArgumentException("Lumen asset payload length does not match its header".toString());
            }
            CRC32 crc32 = new CRC32();
            crc32.update(bytes, unsignedShort2, bytes.length - unsignedShort2);
            if (crc32.getValue() != j) {
                throw new IllegalArgumentException("Lumen asset CRC32 mismatch".toString());
            }
            order.position(unsignedShort2);
            readFloatArray = TractLumenAssetKt.readFloatArray(order, unsignedShort3);
            readFloatArray2 = TractLumenAssetKt.readFloatArray(order, unsignedShort3);
            readFloatArray3 = TractLumenAssetKt.readFloatArray(order, unsignedShort3 * 3);
            readFloatArray4 = TractLumenAssetKt.readFloatArray(order, i3 * 3);
            int[] iArr = new int[i2];
            for (int i4 = 0; i4 < i2; i4++) {
                unsignedShort7 = TractLumenAssetKt.unsignedShort(order);
                iArr[i4] = unsignedShort7;
            }
            if (!order.hasRemaining()) {
                if (unsignedShort5 == 1) {
                    str = "Equal 50/50 male_classical_a and female_classical_a engineering-template blend";
                } else if (unsignedShort5 == 2) {
                    str = "Frozen 0.6.3 metric-MRI engineering mean; five subjects; scientific gate closed";
                } else {
                    throw new IllegalArgumentException("Unsupported lumen provenance " + unsignedShort5);
                }
                hex = TractLumenAssetKt.hex(bArr2);
                hex2 = TractLumenAssetKt.hex(bArr3);
                return new TractLumenAsset(unsignedShort, unsignedShort3, unsignedShort4, readFloatArray, readFloatArray2, readFloatArray3, readFloatArray4, iArr, f, f2, unsignedShort5, str, CollectionsKt.listOf((Object[]) new String[]{hex, hex2}));
            }
            throw new IllegalArgumentException("Unexpected trailing lumen bytes".toString());
        }
    }
}

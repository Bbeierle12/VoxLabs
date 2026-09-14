package org.vocaltract.pixel;

import java.util.Arrays;
import java.util.Collection;
import java.util.Iterator;
import java.util.LinkedHashSet;
import java.util.Set;
import kotlin.Metadata;
import kotlin.collections.ArraysKt;
import kotlin.collections.IntIterator;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: TractMeshCpu.kt */
@Metadata(d1 = {"\u0000J\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\u0014\n\u0002\b\u0004\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0010\u0007\n\u0002\b\u0003\n\u0002\u0010\u0015\n\u0002\b\u0002\n\u0002\u0010\u0002\n\u0000\n\u0002\u0010#\n\u0002\u0010\t\n\u0000\n\u0002\u0010\b\n\u0002\b\u0002\bÀ\u0002\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u001e\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u00052\u0006\u0010\u0007\u001a\u00020\u00052\u0006\u0010\b\u001a\u00020\u0005J\u001e\u0010\t\u001a\u00020\n2\u0006\u0010\u000b\u001a\u00020\f2\u0006\u0010\r\u001a\u00020\u00052\u0006\u0010\u000e\u001a\u00020\u000fJ\u0016\u0010\u0010\u001a\u00020\u00052\u0006\u0010\u0011\u001a\u00020\u00052\u0006\u0010\u0012\u001a\u00020\u0013J\u000e\u0010\u0014\u001a\u00020\u00132\u0006\u0010\u0012\u001a\u00020\u0013J&\u0010\u0015\u001a\u00020\u00162\f\u0010\u0017\u001a\b\u0012\u0004\u0012\u00020\u00190\u00182\u0006\u0010\u001a\u001a\u00020\u001b2\u0006\u0010\u001c\u001a\u00020\u001bH\u0002J \u0010\u001d\u001a\u00020\u00052\u0006\u0010\u000b\u001a\u00020\f2\u0006\u0010\u0011\u001a\u00020\u00052\u0006\u0010\u000e\u001a\u00020\u000fH\u0002"}, d2 = {"Lorg/vocaltract/pixel/TractMeshCpu;", "", "<init>", "()V", "resampleArea", "", "sourcePosition", "sourceAreaCm2", "targetPosition", "prepare", "Lorg/vocaltract/pixel/CpuTractFrame;", "asset", "Lorg/vocaltract/pixel/TractLumenAsset;", "areaCm2", "relativeAreaStd", "", "computeVertexNormals", "vertices", "triangleIndices", "", "buildLineIndices", "addEdge", "", "edges", "", "", "first", "", "second", "expandUncertainty"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class TractMeshCpu {
    public static final TractMeshCpu INSTANCE = new TractMeshCpu();

    private TractMeshCpu() {
    }

    public final float[] resampleArea(float[] sourcePosition, float[] sourceAreaCm2, float[] targetPosition) {
        float f;
        float f2;
        Intrinsics.checkNotNullParameter(sourcePosition, "sourcePosition");
        Intrinsics.checkNotNullParameter(sourceAreaCm2, "sourceAreaCm2");
        Intrinsics.checkNotNullParameter(targetPosition, "targetPosition");
        if (sourcePosition.length != sourceAreaCm2.length) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (!(!(sourcePosition.length == 0))) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        for (float f3 : sourcePosition) {
            if (Float.isInfinite(f3) || Float.isNaN(f3)) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        for (float f4 : sourceAreaCm2) {
            if (Float.isInfinite(f4) || Float.isNaN(f4) || f4 <= 0.0f) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        for (float f5 : targetPosition) {
            if (Float.isInfinite(f5) || Float.isNaN(f5)) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        Iterable until = RangesKt.until(1, sourcePosition.length);
        if (!(until instanceof Collection) || !((Collection) until).isEmpty()) {
            Iterator it = until.iterator();
            while (it.hasNext()) {
                int nextInt = ((IntIterator) it).nextInt();
                if (sourcePosition[nextInt] <= sourcePosition[nextInt - 1]) {
                    throw new IllegalArgumentException("Failed requirement.".toString());
                }
            }
        }
        int length = targetPosition.length;
        float[] fArr = new float[length];
        for (int i = 0; i < length; i++) {
            float f6 = targetPosition[i];
            if (f6 <= ArraysKt.first(sourcePosition)) {
                f2 = ArraysKt.first(sourceAreaCm2);
            } else if (f6 >= ArraysKt.last(sourcePosition)) {
                f2 = ArraysKt.last(sourceAreaCm2);
            } else {
                int i2 = 1;
                while (true) {
                    f = sourcePosition[i2];
                    if (f >= f6) {
                        break;
                    }
                    i2++;
                }
                int i3 = i2 - 1;
                float f7 = sourcePosition[i3];
                float f8 = (f6 - f7) / (f - f7);
                f2 = (sourceAreaCm2[i3] * (1.0f - f8)) + (sourceAreaCm2[i2] * f8);
            }
            fArr[i] = RangesKt.coerceAtLeast(f2, 0.001f);
        }
        return fArr;
    }

    public final CpuTractFrame prepare(TractLumenAsset asset, float[] areaCm2, float relativeAreaStd) {
        Intrinsics.checkNotNullParameter(asset, "asset");
        Intrinsics.checkNotNullParameter(areaCm2, "areaCm2");
        if (areaCm2.length != asset.getSectionCount()) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        float[] morph = asset.morph(areaCm2);
        return new CpuTractFrame(morph, computeVertexNormals(morph, asset.getTriangleIndices()), expandUncertainty(asset, morph, relativeAreaStd));
    }

    public final float[] computeVertexNormals(float[] vertices, int[] triangleIndices) {
        Intrinsics.checkNotNullParameter(vertices, "vertices");
        Intrinsics.checkNotNullParameter(triangleIndices, "triangleIndices");
        if (vertices.length % 3 != 0) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (triangleIndices.length % 3 != 0) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        int length = vertices.length;
        float[] fArr = new float[length];
        for (int i = 0; i < triangleIndices.length; i += 3) {
            int i2 = triangleIndices[i] * 3;
            int i3 = triangleIndices[i + 1] * 3;
            int i4 = triangleIndices[i + 2] * 3;
            if (i2 < 0 || i2 >= vertices.length || i3 < 0 || i3 >= vertices.length || i4 < 0 || i4 >= vertices.length) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            float f = vertices[i3];
            float f2 = vertices[i2];
            float f3 = f - f2;
            float f4 = vertices[i3 + 1];
            float f5 = vertices[i2 + 1];
            float f6 = f4 - f5;
            float f7 = vertices[i3 + 2];
            float f8 = vertices[i2 + 2];
            float f9 = f7 - f8;
            float f10 = vertices[i4] - f2;
            float f11 = vertices[i4 + 1] - f5;
            float f12 = vertices[i4 + 2] - f8;
            float f13 = (f6 * f12) - (f9 * f11);
            float f14 = (f9 * f10) - (f12 * f3);
            float f15 = (f3 * f11) - (f6 * f10);
            int[] iArr = {i2, i3, i4};
            for (int i5 = 0; i5 < 3; i5++) {
                int i6 = iArr[i5];
                fArr[i6] = fArr[i6] + f13;
                int i7 = i6 + 1;
                fArr[i7] = fArr[i7] + f14;
                int i8 = i6 + 2;
                fArr[i8] = fArr[i8] + f15;
            }
        }
        for (int i9 = 0; i9 < length; i9 += 3) {
            float f16 = fArr[i9];
            int i10 = i9 + 1;
            float f17 = fArr[i10];
            int i11 = i9 + 2;
            float f18 = fArr[i11];
            float sqrt = (float) Math.sqrt(Math.max(1.0E-20f, (f16 * f16) + (f17 * f17) + (f18 * f18)));
            fArr[i9] = f16 / sqrt;
            fArr[i10] = f17 / sqrt;
            fArr[i11] = f18 / sqrt;
        }
        return fArr;
    }

    public final int[] buildLineIndices(int[] triangleIndices) {
        Intrinsics.checkNotNullParameter(triangleIndices, "triangleIndices");
        if (triangleIndices.length % 3 != 0) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        LinkedHashSet linkedHashSet = new LinkedHashSet(triangleIndices.length);
        int i = 0;
        for (int i2 = 0; i2 < triangleIndices.length; i2 += 3) {
            LinkedHashSet linkedHashSet2 = linkedHashSet;
            int i3 = i2 + 1;
            addEdge(linkedHashSet2, triangleIndices[i2], triangleIndices[i3]);
            int i4 = triangleIndices[i3];
            int i5 = i2 + 2;
            addEdge(linkedHashSet2, i4, triangleIndices[i5]);
            addEdge(linkedHashSet2, triangleIndices[i5], triangleIndices[i2]);
        }
        int[] iArr = new int[linkedHashSet.size() * 2];
        Iterator it = linkedHashSet.iterator();
        Intrinsics.checkNotNullExpressionValue(it, "iterator(...)");
        while (it.hasNext()) {
            Object next = it.next();
            Intrinsics.checkNotNullExpressionValue(next, "next(...)");
            long longValue = ((Number) next).longValue();
            int i6 = i + 1;
            iArr[i] = (int) (longValue >>> 32);
            i += 2;
            iArr[i6] = (int) longValue;
        }
        return iArr;
    }

    private final void addEdge(Set<Long> edges, int first, int second) {
        edges.add(Long.valueOf((Math.max(first, second) & 4294967295L) | (Math.min(first, second) << 32)));
    }

    private final float[] expandUncertainty(TractLumenAsset asset, float[] vertices, float relativeAreaStd) {
        float[] copyOf = Arrays.copyOf(vertices, vertices.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        float sqrt = (float) Math.sqrt(RangesKt.coerceIn(relativeAreaStd, 0.0f, 1.5f) + 1.0f);
        int vertexCount = asset.getVertexCount();
        for (int i = 0; i < vertexCount; i++) {
            int coerceIn = RangesKt.coerceIn(asset.getSectionForVertex()[i], 0, asset.getSectionCount() - 1) * 3;
            int i2 = i * 3;
            for (int i3 = 0; i3 < 3; i3++) {
                float f = asset.getCenterlineMm()[coerceIn + i3];
                int i4 = i2 + i3;
                copyOf[i4] = f + ((vertices[i4] - f) * sqrt);
            }
        }
        return copyOf;
    }
}

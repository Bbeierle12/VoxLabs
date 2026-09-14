package org.vocaltract.pixel;

import java.util.Arrays;
import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: TractMeshCpu.kt */
@Metadata(d1 = {"\u0000$\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0014\n\u0002\b\r\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u000e\b\u0080\b\u0018\u00002\u00020\u0001B\u001f\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0003¢\u0006\u0004\b\u0006\u0010\u0007J\t\u0010\f\u001a\u00020\u0003HÆ\u0003J\t\u0010\r\u001a\u00020\u0003HÆ\u0003J\t\u0010\u000e\u001a\u00020\u0003HÆ\u0003J'\u0010\u000f\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00032\b\b\u0002\u0010\u0005\u001a\u00020\u0003HÆ\u0001J\u0013\u0010\u0010\u001a\u00020\u00112\b\u0010\u0012\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u0013\u001a\u00020\u0014HÖ\u0001J\t\u0010\u0015\u001a\u00020\u0016HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\b\u0010\tR\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\n\u0010\tR\u0011\u0010\u0005\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000b\u0010\t"}, d2 = {"Lorg/vocaltract/pixel/CpuTractFrame;", "", "vertices", "", "normals", "uncertaintyVertices", "<init>", "([F[F[F)V", "getVertices", "()[F", "getNormals", "getUncertaintyVertices", "component1", "component2", "component3", "copy", "equals", "", "other", "hashCode", "", "toString", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class CpuTractFrame {
    private final float[] normals;
    private final float[] uncertaintyVertices;
    private final float[] vertices;

    public static /* synthetic */ CpuTractFrame copy$default(CpuTractFrame cpuTractFrame, float[] fArr, float[] fArr2, float[] fArr3, int i, Object obj) {
        if ((i & 1) != 0) {
            fArr = cpuTractFrame.vertices;
        }
        if ((i & 2) != 0) {
            fArr2 = cpuTractFrame.normals;
        }
        if ((i & 4) != 0) {
            fArr3 = cpuTractFrame.uncertaintyVertices;
        }
        return cpuTractFrame.copy(fArr, fArr2, fArr3);
    }

    /* renamed from: component1, reason: from getter */
    public final float[] getVertices() {
        return this.vertices;
    }

    /* renamed from: component2, reason: from getter */
    public final float[] getNormals() {
        return this.normals;
    }

    /* renamed from: component3, reason: from getter */
    public final float[] getUncertaintyVertices() {
        return this.uncertaintyVertices;
    }

    public final CpuTractFrame copy(float[] vertices, float[] normals, float[] uncertaintyVertices) {
        Intrinsics.checkNotNullParameter(vertices, "vertices");
        Intrinsics.checkNotNullParameter(normals, "normals");
        Intrinsics.checkNotNullParameter(uncertaintyVertices, "uncertaintyVertices");
        return new CpuTractFrame(vertices, normals, uncertaintyVertices);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof CpuTractFrame)) {
            return false;
        }
        CpuTractFrame cpuTractFrame = (CpuTractFrame) other;
        return Intrinsics.areEqual(this.vertices, cpuTractFrame.vertices) && Intrinsics.areEqual(this.normals, cpuTractFrame.normals) && Intrinsics.areEqual(this.uncertaintyVertices, cpuTractFrame.uncertaintyVertices);
    }

    public int hashCode() {
        return (((Arrays.hashCode(this.vertices) * 31) + Arrays.hashCode(this.normals)) * 31) + Arrays.hashCode(this.uncertaintyVertices);
    }

    public String toString() {
        return "CpuTractFrame(vertices=" + Arrays.toString(this.vertices) + ", normals=" + Arrays.toString(this.normals) + ", uncertaintyVertices=" + Arrays.toString(this.uncertaintyVertices) + ")";
    }

    public CpuTractFrame(float[] vertices, float[] normals, float[] uncertaintyVertices) {
        Intrinsics.checkNotNullParameter(vertices, "vertices");
        Intrinsics.checkNotNullParameter(normals, "normals");
        Intrinsics.checkNotNullParameter(uncertaintyVertices, "uncertaintyVertices");
        this.vertices = vertices;
        this.normals = normals;
        this.uncertaintyVertices = uncertaintyVertices;
    }

    public final float[] getVertices() {
        return this.vertices;
    }

    public final float[] getNormals() {
        return this.normals;
    }

    public final float[] getUncertaintyVertices() {
        return this.uncertaintyVertices;
    }
}

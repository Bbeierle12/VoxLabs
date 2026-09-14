package org.vocaltract.pixel;

import java.util.Arrays;
import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: SharedTractModel.kt */
@Metadata(d1 = {"\u0000$\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0013\n\u0002\b\r\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u000e\b\u0086\b\u0018\u00002\u00020\u0001B\u001f\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0003¢\u0006\u0004\b\u0006\u0010\u0007J\t\u0010\f\u001a\u00020\u0003HÆ\u0003J\t\u0010\r\u001a\u00020\u0003HÆ\u0003J\t\u0010\u000e\u001a\u00020\u0003HÆ\u0003J'\u0010\u000f\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00032\b\b\u0002\u0010\u0005\u001a\u00020\u0003HÆ\u0001J\u0013\u0010\u0010\u001a\u00020\u00112\b\u0010\u0012\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u0013\u001a\u00020\u0014HÖ\u0001J\t\u0010\u0015\u001a\u00020\u0016HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\b\u0010\tR\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\n\u0010\tR\u0011\u0010\u0005\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000b\u0010\t"}, d2 = {"Lorg/vocaltract/pixel/TractResponse;", "", "frequencyHz", "", "db", "peaksHz", "<init>", "([D[D[D)V", "getFrequencyHz", "()[D", "getDb", "getPeaksHz", "component1", "component2", "component3", "copy", "equals", "", "other", "hashCode", "", "toString", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class TractResponse {
    private final double[] db;
    private final double[] frequencyHz;
    private final double[] peaksHz;

    public static /* synthetic */ TractResponse copy$default(TractResponse tractResponse, double[] dArr, double[] dArr2, double[] dArr3, int i, Object obj) {
        if ((i & 1) != 0) {
            dArr = tractResponse.frequencyHz;
        }
        if ((i & 2) != 0) {
            dArr2 = tractResponse.db;
        }
        if ((i & 4) != 0) {
            dArr3 = tractResponse.peaksHz;
        }
        return tractResponse.copy(dArr, dArr2, dArr3);
    }

    /* renamed from: component1, reason: from getter */
    public final double[] getFrequencyHz() {
        return this.frequencyHz;
    }

    /* renamed from: component2, reason: from getter */
    public final double[] getDb() {
        return this.db;
    }

    /* renamed from: component3, reason: from getter */
    public final double[] getPeaksHz() {
        return this.peaksHz;
    }

    public final TractResponse copy(double[] frequencyHz, double[] db, double[] peaksHz) {
        Intrinsics.checkNotNullParameter(frequencyHz, "frequencyHz");
        Intrinsics.checkNotNullParameter(db, "db");
        Intrinsics.checkNotNullParameter(peaksHz, "peaksHz");
        return new TractResponse(frequencyHz, db, peaksHz);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof TractResponse)) {
            return false;
        }
        TractResponse tractResponse = (TractResponse) other;
        return Intrinsics.areEqual(this.frequencyHz, tractResponse.frequencyHz) && Intrinsics.areEqual(this.db, tractResponse.db) && Intrinsics.areEqual(this.peaksHz, tractResponse.peaksHz);
    }

    public int hashCode() {
        return (((Arrays.hashCode(this.frequencyHz) * 31) + Arrays.hashCode(this.db)) * 31) + Arrays.hashCode(this.peaksHz);
    }

    public String toString() {
        return "TractResponse(frequencyHz=" + Arrays.toString(this.frequencyHz) + ", db=" + Arrays.toString(this.db) + ", peaksHz=" + Arrays.toString(this.peaksHz) + ")";
    }

    public TractResponse(double[] frequencyHz, double[] db, double[] peaksHz) {
        Intrinsics.checkNotNullParameter(frequencyHz, "frequencyHz");
        Intrinsics.checkNotNullParameter(db, "db");
        Intrinsics.checkNotNullParameter(peaksHz, "peaksHz");
        this.frequencyHz = frequencyHz;
        this.db = db;
        this.peaksHz = peaksHz;
    }

    public final double[] getDb() {
        return this.db;
    }

    public final double[] getFrequencyHz() {
        return this.frequencyHz;
    }

    public final double[] getPeaksHz() {
        return this.peaksHz;
    }
}

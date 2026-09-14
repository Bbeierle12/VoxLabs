package org.vocaltract.pixel;

import java.util.Collection;
import java.util.Iterator;
import java.util.List;
import kotlin.Metadata;
import kotlin.collections.CollectionsKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;
import kotlin.text.StringsKt;

/* compiled from: DiagnosticsCore.kt */
@Metadata(d1 = {"\u0000,\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0007\n\u0000\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010 \n\u0002\b\u0002\n\u0002\u0010\u000b\n\u0002\b\u0017\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001BC\u0012\n\b\u0002\u0010\u0002\u001a\u0004\u0018\u00010\u0003\u0012\n\b\u0002\u0010\u0004\u001a\u0004\u0018\u00010\u0005\u0012\u000e\b\u0002\u0010\u0006\u001a\b\u0012\u0004\u0012\u00020\u00030\u0007\u0012\b\b\u0002\u0010\b\u001a\u00020\u0005\u0012\b\b\u0002\u0010\t\u001a\u00020\n¢\u0006\u0004\b\u000b\u0010\fJ\u0006\u0010\u0017\u001a\u00020\u0000J\u0010\u0010\u0018\u001a\u0004\u0018\u00010\u0003HÆ\u0003¢\u0006\u0002\u0010\u000eJ\u000b\u0010\u0019\u001a\u0004\u0018\u00010\u0005HÆ\u0003J\u000f\u0010\u001a\u001a\b\u0012\u0004\u0012\u00020\u00030\u0007HÆ\u0003J\t\u0010\u001b\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001c\u001a\u00020\nHÆ\u0003JJ\u0010\u001d\u001a\u00020\u00002\n\b\u0002\u0010\u0002\u001a\u0004\u0018\u00010\u00032\n\b\u0002\u0010\u0004\u001a\u0004\u0018\u00010\u00052\u000e\b\u0002\u0010\u0006\u001a\b\u0012\u0004\u0012\u00020\u00030\u00072\b\b\u0002\u0010\b\u001a\u00020\u00052\b\b\u0002\u0010\t\u001a\u00020\nHÆ\u0001¢\u0006\u0002\u0010\u001eJ\u0013\u0010\u001f\u001a\u00020\n2\b\u0010 \u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010!\u001a\u00020\"HÖ\u0001J\t\u0010#\u001a\u00020\u0005HÖ\u0001R\u0015\u0010\u0002\u001a\u0004\u0018\u00010\u0003¢\u0006\n\n\u0002\u0010\u000f\u001a\u0004\b\r\u0010\u000eR\u0013\u0010\u0004\u001a\u0004\u0018\u00010\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0010\u0010\u0011R\u0017\u0010\u0006\u001a\b\u0012\u0004\u0012\u00020\u00030\u0007¢\u0006\b\n\u0000\u001a\u0004\b\u0012\u0010\u0013R\u0011\u0010\b\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\u0011R\u0011\u0010\t\u001a\u00020\n¢\u0006\b\n\u0000\u001a\u0004\b\u0015\u0010\u0016"}, d2 = {"Lorg/vocaltract/pixel/RefinementDraft;", "", "expectedF0Hz", "", "expectedVowel", "", "expectedResonancesHz", "", "notes", "approvedForFutureTraining", "", "<init>", "(Ljava/lang/Float;Ljava/lang/String;Ljava/util/List;Ljava/lang/String;Z)V", "getExpectedF0Hz", "()Ljava/lang/Float;", "Ljava/lang/Float;", "getExpectedVowel", "()Ljava/lang/String;", "getExpectedResonancesHz", "()Ljava/util/List;", "getNotes", "getApprovedForFutureTraining", "()Z", "validated", "component1", "component2", "component3", "component4", "component5", "copy", "(Ljava/lang/Float;Ljava/lang/String;Ljava/util/List;Ljava/lang/String;Z)Lorg/vocaltract/pixel/RefinementDraft;", "equals", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class RefinementDraft {
    private final boolean approvedForFutureTraining;
    private final Float expectedF0Hz;
    private final List<Float> expectedResonancesHz;
    private final String expectedVowel;
    private final String notes;

    public RefinementDraft() {
        this(null, null, null, null, false, 31, null);
    }

    public static /* synthetic */ RefinementDraft copy$default(RefinementDraft refinementDraft, Float f, String str, List list, String str2, boolean z, int i, Object obj) {
        if ((i & 1) != 0) {
            f = refinementDraft.expectedF0Hz;
        }
        if ((i & 2) != 0) {
            str = refinementDraft.expectedVowel;
        }
        String str3 = str;
        if ((i & 4) != 0) {
            list = refinementDraft.expectedResonancesHz;
        }
        List list2 = list;
        if ((i & 8) != 0) {
            str2 = refinementDraft.notes;
        }
        String str4 = str2;
        if ((i & 16) != 0) {
            z = refinementDraft.approvedForFutureTraining;
        }
        return refinementDraft.copy(f, str3, list2, str4, z);
    }

    /* renamed from: component1, reason: from getter */
    public final Float getExpectedF0Hz() {
        return this.expectedF0Hz;
    }

    /* renamed from: component2, reason: from getter */
    public final String getExpectedVowel() {
        return this.expectedVowel;
    }

    public final List<Float> component3() {
        return this.expectedResonancesHz;
    }

    /* renamed from: component4, reason: from getter */
    public final String getNotes() {
        return this.notes;
    }

    /* renamed from: component5, reason: from getter */
    public final boolean getApprovedForFutureTraining() {
        return this.approvedForFutureTraining;
    }

    public final RefinementDraft copy(Float expectedF0Hz, String expectedVowel, List<Float> expectedResonancesHz, String notes, boolean approvedForFutureTraining) {
        Intrinsics.checkNotNullParameter(expectedResonancesHz, "expectedResonancesHz");
        Intrinsics.checkNotNullParameter(notes, "notes");
        return new RefinementDraft(expectedF0Hz, expectedVowel, expectedResonancesHz, notes, approvedForFutureTraining);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof RefinementDraft)) {
            return false;
        }
        RefinementDraft refinementDraft = (RefinementDraft) other;
        return Intrinsics.areEqual((Object) this.expectedF0Hz, (Object) refinementDraft.expectedF0Hz) && Intrinsics.areEqual(this.expectedVowel, refinementDraft.expectedVowel) && Intrinsics.areEqual(this.expectedResonancesHz, refinementDraft.expectedResonancesHz) && Intrinsics.areEqual(this.notes, refinementDraft.notes) && this.approvedForFutureTraining == refinementDraft.approvedForFutureTraining;
    }

    public int hashCode() {
        Float f = this.expectedF0Hz;
        int hashCode = (f == null ? 0 : f.hashCode()) * 31;
        String str = this.expectedVowel;
        return ((((((hashCode + (str != null ? str.hashCode() : 0)) * 31) + this.expectedResonancesHz.hashCode()) * 31) + this.notes.hashCode()) * 31) + Boolean.hashCode(this.approvedForFutureTraining);
    }

    public String toString() {
        return "RefinementDraft(expectedF0Hz=" + this.expectedF0Hz + ", expectedVowel=" + this.expectedVowel + ", expectedResonancesHz=" + this.expectedResonancesHz + ", notes=" + this.notes + ", approvedForFutureTraining=" + this.approvedForFutureTraining + ")";
    }

    public RefinementDraft(Float f, String str, List<Float> expectedResonancesHz, String notes, boolean z) {
        Intrinsics.checkNotNullParameter(expectedResonancesHz, "expectedResonancesHz");
        Intrinsics.checkNotNullParameter(notes, "notes");
        this.expectedF0Hz = f;
        this.expectedVowel = str;
        this.expectedResonancesHz = expectedResonancesHz;
        this.notes = notes;
        this.approvedForFutureTraining = z;
    }

    public final Float getExpectedF0Hz() {
        return this.expectedF0Hz;
    }

    public final String getExpectedVowel() {
        return this.expectedVowel;
    }

    public /* synthetic */ RefinementDraft(Float f, String str, List list, String str2, boolean z, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this((i & 1) != 0 ? null : f, (i & 2) != 0 ? null : str, (i & 4) != 0 ? CollectionsKt.emptyList() : list, (i & 8) != 0 ? "" : str2, (i & 16) != 0 ? false : z);
    }

    public final List<Float> getExpectedResonancesHz() {
        return this.expectedResonancesHz;
    }

    public final String getNotes() {
        return this.notes;
    }

    public final boolean getApprovedForFutureTraining() {
        return this.approvedForFutureTraining;
    }

    public final RefinementDraft validated() {
        String obj;
        String str;
        Float f = this.expectedF0Hz;
        if (f != null) {
            float floatValue = f.floatValue();
            if (Float.isInfinite(floatValue) || Float.isNaN(floatValue) || !RangesKt.rangeTo(40.0f, 2000.0f).contains(this.expectedF0Hz)) {
                throw new IllegalArgumentException("Expected F0 must be between 40 and 2000 Hz".toString());
            }
        }
        if (this.expectedResonancesHz.size() > 6) {
            throw new IllegalArgumentException("At most six expected resonances are supported".toString());
        }
        List<Float> list = this.expectedResonancesHz;
        if (!(list instanceof Collection) || !list.isEmpty()) {
            Iterator<T> it = list.iterator();
            while (it.hasNext()) {
                float floatValue2 = ((Number) it.next()).floatValue();
                if (Float.isInfinite(floatValue2) || Float.isNaN(floatValue2) || 80.0f > floatValue2 || floatValue2 > 10000.0f) {
                    throw new IllegalArgumentException("Expected resonances must be between 80 and 10000 Hz".toString());
                }
            }
        }
        if (this.expectedF0Hz == null && (((str = this.expectedVowel) == null || StringsKt.isBlank(str)) && !(!this.expectedResonancesHz.isEmpty()) && !(!StringsKt.isBlank(this.notes)))) {
            throw new IllegalArgumentException("A correction needs an expected value or a note".toString());
        }
        String str2 = this.expectedVowel;
        return copy$default(this, null, (str2 == null || (obj = StringsKt.trim((CharSequence) str2).toString()) == null || obj.length() <= 0) ? null : obj, null, StringsKt.trim((CharSequence) this.notes).toString(), false, 21, null);
    }
}

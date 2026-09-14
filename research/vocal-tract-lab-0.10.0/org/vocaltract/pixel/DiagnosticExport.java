package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: DiagnosticStore.kt */
@Metadata(d1 = {"\u0000\"\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000e\n\u0002\b\u0002\n\u0002\u0010\b\n\u0002\b\f\n\u0002\u0010\u000b\n\u0002\b\u0003\b\u0086\b\u0018\u00002\u00020\u0001B\u001f\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0006¢\u0006\u0004\b\u0007\u0010\bJ\t\u0010\u000e\u001a\u00020\u0003HÆ\u0003J\t\u0010\u000f\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0010\u001a\u00020\u0006HÆ\u0003J'\u0010\u0011\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00032\b\b\u0002\u0010\u0005\u001a\u00020\u0006HÆ\u0001J\u0013\u0010\u0012\u001a\u00020\u00132\b\u0010\u0014\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u0015\u001a\u00020\u0006HÖ\u0001J\t\u0010\u0016\u001a\u00020\u0003HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\t\u0010\nR\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000b\u0010\nR\u0011\u0010\u0005\u001a\u00020\u0006¢\u0006\b\n\u0000\u001a\u0004\b\f\u0010\r"}, d2 = {"Lorg/vocaltract/pixel/DiagnosticExport;", "", "displayName", "", "uri", "bytes", "", "<init>", "(Ljava/lang/String;Ljava/lang/String;I)V", "getDisplayName", "()Ljava/lang/String;", "getUri", "getBytes", "()I", "component1", "component2", "component3", "copy", "equals", "", "other", "hashCode", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class DiagnosticExport {
    private final int bytes;
    private final String displayName;
    private final String uri;

    public static /* synthetic */ DiagnosticExport copy$default(DiagnosticExport diagnosticExport, String str, String str2, int i, int i2, Object obj) {
        if ((i2 & 1) != 0) {
            str = diagnosticExport.displayName;
        }
        if ((i2 & 2) != 0) {
            str2 = diagnosticExport.uri;
        }
        if ((i2 & 4) != 0) {
            i = diagnosticExport.bytes;
        }
        return diagnosticExport.copy(str, str2, i);
    }

    /* renamed from: component1, reason: from getter */
    public final String getDisplayName() {
        return this.displayName;
    }

    /* renamed from: component2, reason: from getter */
    public final String getUri() {
        return this.uri;
    }

    /* renamed from: component3, reason: from getter */
    public final int getBytes() {
        return this.bytes;
    }

    public final DiagnosticExport copy(String displayName, String uri, int bytes) {
        Intrinsics.checkNotNullParameter(displayName, "displayName");
        Intrinsics.checkNotNullParameter(uri, "uri");
        return new DiagnosticExport(displayName, uri, bytes);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof DiagnosticExport)) {
            return false;
        }
        DiagnosticExport diagnosticExport = (DiagnosticExport) other;
        return Intrinsics.areEqual(this.displayName, diagnosticExport.displayName) && Intrinsics.areEqual(this.uri, diagnosticExport.uri) && this.bytes == diagnosticExport.bytes;
    }

    public int hashCode() {
        return (((this.displayName.hashCode() * 31) + this.uri.hashCode()) * 31) + Integer.hashCode(this.bytes);
    }

    public String toString() {
        return "DiagnosticExport(displayName=" + this.displayName + ", uri=" + this.uri + ", bytes=" + this.bytes + ")";
    }

    public DiagnosticExport(String displayName, String uri, int i) {
        Intrinsics.checkNotNullParameter(displayName, "displayName");
        Intrinsics.checkNotNullParameter(uri, "uri");
        this.displayName = displayName;
        this.uri = uri;
        this.bytes = i;
    }

    public final String getDisplayName() {
        return this.displayName;
    }

    public final String getUri() {
        return this.uri;
    }

    public final int getBytes() {
        return this.bytes;
    }
}

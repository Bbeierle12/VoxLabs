package org.vocaltract.pixel;

import java.util.Iterator;
import java.util.Set;
import kotlin.Metadata;
import kotlin.collections.SetsKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.sequences.SequencesKt;
import org.json.JSONObject;

/* compiled from: SharedLabJson.kt */
@Metadata(d1 = {"\u0000&\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u0006\n\u0002\b\u0017\n\u0002\u0010\u000b\n\u0002\b\u0004\b\u0086\b\u0018\u0000 #2\u00020\u0001:\u0001#B=\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\n\b\u0002\u0010\u0004\u001a\u0004\u0018\u00010\u0005\u0012\n\b\u0002\u0010\u0006\u001a\u0004\u0018\u00010\u0007\u0012\n\b\u0002\u0010\b\u001a\u0004\u0018\u00010\u0003\u0012\b\b\u0002\u0010\t\u001a\u00020\u0005¢\u0006\u0004\b\n\u0010\u000bJ\t\u0010\u0017\u001a\u00020\u0003HÆ\u0003J\u0010\u0010\u0018\u001a\u0004\u0018\u00010\u0005HÆ\u0003¢\u0006\u0002\u0010\u000fJ\u0010\u0010\u0019\u001a\u0004\u0018\u00010\u0007HÆ\u0003¢\u0006\u0002\u0010\u0012J\u000b\u0010\u001a\u001a\u0004\u0018\u00010\u0003HÆ\u0003J\t\u0010\u001b\u001a\u00020\u0005HÆ\u0003JF\u0010\u001c\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\n\b\u0002\u0010\u0004\u001a\u0004\u0018\u00010\u00052\n\b\u0002\u0010\u0006\u001a\u0004\u0018\u00010\u00072\n\b\u0002\u0010\b\u001a\u0004\u0018\u00010\u00032\b\b\u0002\u0010\t\u001a\u00020\u0005HÆ\u0001¢\u0006\u0002\u0010\u001dJ\u0013\u0010\u001e\u001a\u00020\u001f2\b\u0010 \u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010!\u001a\u00020\u0005HÖ\u0001J\t\u0010\"\u001a\u00020\u0003HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\f\u0010\rR\u0015\u0010\u0004\u001a\u0004\u0018\u00010\u0005¢\u0006\n\n\u0002\u0010\u0010\u001a\u0004\b\u000e\u0010\u000fR\u0015\u0010\u0006\u001a\u0004\u0018\u00010\u0007¢\u0006\n\n\u0002\u0010\u0013\u001a\u0004\b\u0011\u0010\u0012R\u0013\u0010\b\u001a\u0004\u0018\u00010\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\rR\u0011\u0010\t\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0015\u0010\u0016"}, d2 = {"Lorg/vocaltract/pixel/LabCommand;", "", "type", "", "index", "", "value", "", "id", "requestId", "<init>", "(Ljava/lang/String;Ljava/lang/Integer;Ljava/lang/Double;Ljava/lang/String;I)V", "getType", "()Ljava/lang/String;", "getIndex", "()Ljava/lang/Integer;", "Ljava/lang/Integer;", "getValue", "()Ljava/lang/Double;", "Ljava/lang/Double;", "getId", "getRequestId", "()I", "component1", "component2", "component3", "component4", "component5", "copy", "(Ljava/lang/String;Ljava/lang/Integer;Ljava/lang/Double;Ljava/lang/String;I)Lorg/vocaltract/pixel/LabCommand;", "equals", "", "other", "hashCode", "toString", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class LabCommand {

    /* renamed from: Companion, reason: from kotlin metadata */
    public static final Companion INSTANCE = new Companion(null);
    private final String id;
    private final Integer index;
    private final int requestId;
    private final String type;
    private final Double value;

    public static /* synthetic */ LabCommand copy$default(LabCommand labCommand, String str, Integer num, Double d, String str2, int i, int i2, Object obj) {
        if ((i2 & 1) != 0) {
            str = labCommand.type;
        }
        if ((i2 & 2) != 0) {
            num = labCommand.index;
        }
        Integer num2 = num;
        if ((i2 & 4) != 0) {
            d = labCommand.value;
        }
        Double d2 = d;
        if ((i2 & 8) != 0) {
            str2 = labCommand.id;
        }
        String str3 = str2;
        if ((i2 & 16) != 0) {
            i = labCommand.requestId;
        }
        return labCommand.copy(str, num2, d2, str3, i);
    }

    /* renamed from: component1, reason: from getter */
    public final String getType() {
        return this.type;
    }

    /* renamed from: component2, reason: from getter */
    public final Integer getIndex() {
        return this.index;
    }

    /* renamed from: component3, reason: from getter */
    public final Double getValue() {
        return this.value;
    }

    /* renamed from: component4, reason: from getter */
    public final String getId() {
        return this.id;
    }

    /* renamed from: component5, reason: from getter */
    public final int getRequestId() {
        return this.requestId;
    }

    public final LabCommand copy(String type, Integer index, Double value, String id, int requestId) {
        Intrinsics.checkNotNullParameter(type, "type");
        return new LabCommand(type, index, value, id, requestId);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof LabCommand)) {
            return false;
        }
        LabCommand labCommand = (LabCommand) other;
        return Intrinsics.areEqual(this.type, labCommand.type) && Intrinsics.areEqual(this.index, labCommand.index) && Intrinsics.areEqual((Object) this.value, (Object) labCommand.value) && Intrinsics.areEqual(this.id, labCommand.id) && this.requestId == labCommand.requestId;
    }

    public int hashCode() {
        int hashCode = this.type.hashCode() * 31;
        Integer num = this.index;
        int hashCode2 = (hashCode + (num == null ? 0 : num.hashCode())) * 31;
        Double d = this.value;
        int hashCode3 = (hashCode2 + (d == null ? 0 : d.hashCode())) * 31;
        String str = this.id;
        return ((hashCode3 + (str != null ? str.hashCode() : 0)) * 31) + Integer.hashCode(this.requestId);
    }

    public String toString() {
        return "LabCommand(type=" + this.type + ", index=" + this.index + ", value=" + this.value + ", id=" + this.id + ", requestId=" + this.requestId + ")";
    }

    /* compiled from: SharedLabJson.kt */
    @Metadata(d1 = {"\u0000\u0016\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u000e\b\u0086\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u000e\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u0007"}, d2 = {"Lorg/vocaltract/pixel/LabCommand$Companion;", "", "<init>", "()V", "parse", "Lorg/vocaltract/pixel/LabCommand;", "raw", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
    public static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }

        /* JADX DEBUG: Don't trust debug lines info. Repeating lines: [58=4] */
        /* JADX WARN: Can't fix incorrect switch cases order, some code will duplicate */
        /* JADX WARN: Code restructure failed: missing block: B:68:0x003a, code lost:
        
            if (r3.equals("reset") != false) goto L53;
         */
        /* JADX WARN: Code restructure failed: missing block: B:69:0x00cd, code lost:
        
            r14 = kotlin.collections.SetsKt.setOf("type");
         */
        /* JADX WARN: Code restructure failed: missing block: B:71:0x0044, code lost:
        
            if (r3.equals("ready") != false) goto L53;
         */
        /* JADX WARN: Code restructure failed: missing block: B:73:0x004e, code lost:
        
            if (r3.equals("pitch") != false) goto L47;
         */
        /* JADX WARN: Code restructure failed: missing block: B:74:0x00b3, code lost:
        
            r14 = kotlin.collections.SetsKt.setOf((java.lang.Object[]) new java.lang.String[]{"type", "value"});
         */
        /* JADX WARN: Code restructure failed: missing block: B:76:0x0057, code lost:
        
            if (r3.equals("noise") != false) goto L53;
         */
        /* JADX WARN: Code restructure failed: missing block: B:78:0x0061, code lost:
        
            if (r3.equals("stop") != false) goto L53;
         */
        /* JADX WARN: Code restructure failed: missing block: B:80:0x006b, code lost:
        
            if (r3.equals("play") != false) goto L53;
         */
        /* JADX WARN: Code restructure failed: missing block: B:82:0x0074, code lost:
        
            if (r3.equals("demo") != false) goto L53;
         */
        /* JADX WARN: Code restructure failed: missing block: B:84:0x007d, code lost:
        
            if (r3.equals("formant") != false) goto L38;
         */
        /* JADX WARN: Code restructure failed: missing block: B:86:0x0086, code lost:
        
            if (r3.equals("partial") != false) goto L38;
         */
        /* JADX WARN: Code restructure failed: missing block: B:88:0x0097, code lost:
        
            if (r3.equals("retune") != false) goto L53;
         */
        /* JADX WARN: Code restructure failed: missing block: B:8:0x0031, code lost:
        
            if (r3.equals("coordinate") != false) goto L38;
         */
        /* JADX WARN: Code restructure failed: missing block: B:93:0x00b1, code lost:
        
            if (r3.equals("master") != false) goto L47;
         */
        /* JADX WARN: Code restructure failed: missing block: B:95:0x00c2, code lost:
        
            if (r3.equals("listen") != false) goto L53;
         */
        /* JADX WARN: Code restructure failed: missing block: B:97:0x00cb, code lost:
        
            if (r3.equals("calibrate") != false) goto L53;
         */
        /* JADX WARN: Code restructure failed: missing block: B:9:0x0088, code lost:
        
            r14 = kotlin.collections.SetsKt.setOf((java.lang.Object[]) new java.lang.String[]{"type", "index", "value"});
         */
        /* JADX WARN: Failed to restore switch over string. Please report as a decompilation issue */
        /*
            Code decompiled incorrectly, please refer to instructions dump.
        */
        public final LabCommand parse(String raw) {
            Integer num;
            Double d;
            Intrinsics.checkNotNullParameter(raw, "raw");
            if (raw.length() > 1024) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            JSONObject jSONObject = new JSONObject(raw);
            String string = jSONObject.getString("type");
            if (string != null) {
                switch (string.hashCode()) {
                    case -1129246009:
                        break;
                    case -1102508601:
                        break;
                    case -1081267614:
                        break;
                    case -980098337:
                        if (string.equals("preset")) {
                            Set of = SetsKt.setOf((Object[]) new String[]{"type", "id"});
                            Iterator<String> keys = jSONObject.keys();
                            Intrinsics.checkNotNullExpressionValue(keys, "keys(...)");
                            if (!Intrinsics.areEqual(SequencesKt.toSet(SequencesKt.asSequence(keys)), SetsKt.plus((Set<? extends String>) of, "requestId"))) {
                                throw new IllegalArgumentException("Failed requirement.".toString());
                            }
                            if (!(jSONObject.get("requestId") instanceof Number)) {
                                throw new IllegalArgumentException("Failed requirement.".toString());
                            }
                            double d2 = jSONObject.getDouble("requestId");
                            if (Double.isInfinite(d2) || Double.isNaN(d2) || 1.0d > d2 || d2 > 2.147483647E9d || d2 != ((int) d2)) {
                                throw new IllegalArgumentException("Failed requirement.".toString());
                            }
                            if (!of.contains("index")) {
                                num = null;
                            } else {
                                if (!(jSONObject.get("index") instanceof Number)) {
                                    throw new IllegalArgumentException("Failed requirement.".toString());
                                }
                                double d3 = jSONObject.getDouble("index");
                                if (Double.isInfinite(d3) || Double.isNaN(d3) || d3 != ((int) d3)) {
                                    throw new IllegalArgumentException("Failed requirement.".toString());
                                }
                                num = Integer.valueOf((int) d3);
                            }
                            if (!of.contains("value")) {
                                d = null;
                            } else {
                                if (!(jSONObject.get("value") instanceof Number)) {
                                    throw new IllegalArgumentException("Failed requirement.".toString());
                                }
                                Double valueOf = Double.valueOf(jSONObject.getDouble("value"));
                                double doubleValue = valueOf.doubleValue();
                                if (Double.isInfinite(doubleValue) || Double.isNaN(doubleValue)) {
                                    throw new IllegalArgumentException("Failed requirement.".toString());
                                }
                                d = valueOf;
                            }
                            return new LabCommand(string, num, d, of.contains("id") ? jSONObject.getString("id") : null, (int) d2);
                        }
                        break;
                    case -934396757:
                        break;
                    case -792934015:
                        break;
                    case -677443933:
                        break;
                    case 3079651:
                        break;
                    case 3443508:
                        break;
                    case 3540994:
                        break;
                    case 104998682:
                        break;
                    case 106677056:
                        break;
                    case 108386723:
                        break;
                    case 108404047:
                        break;
                    case 198931832:
                        break;
                }
            }
            throw new IllegalStateException("Unknown lab command".toString());
        }
    }

    public LabCommand(String type, Integer num, Double d, String str, int i) {
        Intrinsics.checkNotNullParameter(type, "type");
        this.type = type;
        this.index = num;
        this.value = d;
        this.id = str;
        this.requestId = i;
    }

    public /* synthetic */ LabCommand(String str, Integer num, Double d, String str2, int i, int i2, DefaultConstructorMarker defaultConstructorMarker) {
        this(str, (i2 & 2) != 0 ? null : num, (i2 & 4) != 0 ? null : d, (i2 & 8) != 0 ? null : str2, (i2 & 16) != 0 ? 0 : i);
    }

    public final String getId() {
        return this.id;
    }

    public final Integer getIndex() {
        return this.index;
    }

    public final int getRequestId() {
        return this.requestId;
    }

    public final String getType() {
        return this.type;
    }

    public final Double getValue() {
        return this.value;
    }
}

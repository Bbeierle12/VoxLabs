package org.vocaltract.pixel;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import kotlin.Metadata;
import kotlin.Triple;
import kotlin.TuplesKt;
import kotlin.collections.ArraysKt;
import kotlin.collections.CollectionsKt;
import kotlin.collections.MapsKt;
import kotlin.jvm.internal.Intrinsics;
import org.json.JSONArray;
import org.json.JSONObject;

/* compiled from: SharedLabJson.kt */
@Metadata(d1 = {"\u0000H\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010 \n\u0002\u0010\u0006\n\u0000\n\u0002\u0010\u0014\n\u0000\n\u0002\u0010\u0013\n\u0000\n\u0002\u0018\u0002\n\u0002\u0010\u000e\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010$\n\u0002\b\u0002\n\u0002\u0010\u000b\n\u0000\bÆ\u0002\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u0016\u0010\u0004\u001a\b\u0012\u0004\u0012\u00020\u00060\u00052\u0006\u0010\u0007\u001a\u00020\bH\u0002J\u0016\u0010\t\u001a\b\u0012\u0004\u0012\u00020\u00060\u00052\u0006\u0010\u0007\u001a\u00020\nH\u0002J\u000e\u0010\u0010\u001a\u00020\u00112\u0006\u0010\u0012\u001a\u00020\u0013J\u001c\u0010\u0014\u001a\u0010\u0012\u0004\u0012\u00020\r\u0012\u0006\u0012\u0004\u0018\u00010\u00010\u00152\u0006\u0010\u0012\u001a\u00020\u0013J\u000e\u0010\u0016\u001a\u00020\r2\u0006\u0010\u0012\u001a\u00020\u0013J\u0016\u0010\u0017\u001a\u00020\u00182\u0006\u0010\u0012\u001a\u00020\u00132\u0006\u0010\u0019\u001a\u00020\rR)\u0010\u000b\u001a\u001a\u0012\u0016\u0012\u0014\u0012\u0004\u0012\u00020\r\u0012\u0004\u0012\u00020\r\u0012\u0004\u0012\u00020\b0\f0\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u000e\u0010\u000f"}, d2 = {"Lorg/vocaltract/pixel/SharedLabJson;", "", "<init>", "()V", "floats", "", "", "a", "", "doubles", "", "presets", "Lkotlin/Triple;", "", "getPresets", "()Ljava/util/List;", "snapshot", "Lorg/json/JSONObject;", "state", "Lorg/vocaltract/pixel/SharedLabState;", "snapshotValues", "", "save", "restore", "", "raw"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class SharedLabJson {
    public static final SharedLabJson INSTANCE = new SharedLabJson();
    private static final List<Triple<String, String, float[]>> presets = CollectionsKt.listOf((Object[]) new Triple[]{new Triple("neutral", "Neutral /ə/", new float[]{500.0f, 1500.0f, 2500.0f}), new Triple("open_a", "Open A /ɑ/", new float[]{730.0f, 1090.0f, 2440.0f}), new Triple("front_a", "Front A /æ/", new float[]{660.0f, 1720.0f, 2410.0f}), new Triple("ee", "EE /i/", new float[]{270.0f, 2290.0f, 3010.0f}), new Triple("oh", "OH /o/", new float[]{570.0f, 840.0f, 2410.0f}), new Triple("oo", "OO /u/", new float[]{300.0f, 870.0f, 2240.0f})});

    private SharedLabJson() {
    }

    private final List<Double> doubles(double[] a) {
        return ArraysKt.toList(a);
    }

    public final List<Triple<String, String, float[]>> getPresets() {
        return presets;
    }

    public final JSONObject snapshot(SharedLabState state) {
        Intrinsics.checkNotNullParameter(state, "state");
        return new JSONObject(snapshotValues(state));
    }

    public final Map<String, Object> snapshotValues(SharedLabState state) {
        List<Double> list;
        Intrinsics.checkNotNullParameter(state, "state");
        SharedTractModel model = state.getModel();
        float[] coefficients = state.getCoefficients();
        float[] area = model.area(coefficients);
        TractResponse response = model.response(coefficients);
        AdditiveSynthState sound = state.sound();
        float[] targetAmplitudes = AdditiveSynthesisMath.INSTANCE.targetAmplitudes(sound.withPlaying(true), SummedWaveformView.DISPLAY_SAMPLE_RATE_HZ);
        LinkedHashMap linkedHashMap = new LinkedHashMap();
        linkedHashMap.put("schema", "shared-mri-lab-v1");
        linkedHashMap.put("modelId", model.getId());
        linkedHashMap.put("revision", Long.valueOf(state.getRevision()));
        linkedHashMap.put("atlasId", model.getAtlas().getModelId());
        linkedHashMap.put("sourceModelSha256", model.getAtlas().getAtlasSourceModelSha256());
        linkedHashMap.put("freezeManifestSha256", model.getAtlas().getAtlasFreezeManifestSha256());
        linkedHashMap.put("subjects", Integer.valueOf(model.getAtlas().getAtlasSubjects()));
        linkedHashMap.put("expertAcceptances", 0);
        linkedHashMap.put("scientificReleaseReady", false);
        linkedHashMap.put("origin", state.getOrigin());
        linkedHashMap.put("coefficients", INSTANCE.floats(coefficients));
        float[][] coefficientLimitsSd = model.getAtlas().getCoefficientLimitsSd();
        ArrayList arrayList = new ArrayList(coefficientLimitsSd.length);
        for (float[] fArr : coefficientLimitsSd) {
            arrayList.add(INSTANCE.floats(fArr));
        }
        linkedHashMap.put("limits", arrayList);
        linkedHashMap.put("modeLabels", model.getAtlas().getModeLabels());
        linkedHashMap.put("clampedSections", Integer.valueOf(model.clampedSections(coefficients)));
        SharedLabJson sharedLabJson = INSTANCE;
        linkedHashMap.put("vertices", sharedLabJson.floats(model.vertices(coefficients)));
        linkedHashMap.put("faces", ArraysKt.toList(model.getLumen().getTriangleIndices()));
        linkedHashMap.put("centers", sharedLabJson.floats(model.getLumen().getCenterlineMm()));
        linkedHashMap.put("sections", Integer.valueOf(model.getLumen().getSectionCount()));
        linkedHashMap.put("angles", Integer.valueOf(model.getLumen().getAngularSamples()));
        linkedHashMap.put("areas", sharedLabJson.floats(area));
        linkedHashMap.put("lengths", sharedLabJson.doubles(model.getLengthsMm()));
        linkedHashMap.put("lengthMm", Double.valueOf(model.getLengthMm()));
        linkedHashMap.put("responseFrequency", sharedLabJson.doubles(response.getFrequencyHz()));
        linkedHashMap.put("responseDb", sharedLabJson.doubles(response.getDb()));
        linkedHashMap.put("resonances", sharedLabJson.doubles(response.getPeaksHz()));
        linkedHashMap.put("f0", Double.valueOf(state.getF0()));
        linkedHashMap.put("master", Double.valueOf(state.getMaster()));
        linkedHashMap.put("sourceGains", sharedLabJson.floats(state.getSourceGains()));
        linkedHashMap.put("outputGains", sharedLabJson.floats(CollectionsKt.toFloatArray(sound.partialGains())));
        linkedHashMap.put("amplitudes", sharedLabJson.floats(targetAmplitudes));
        linkedHashMap.put("playing", false);
        float[] targets = state.getTargets();
        Double d = null;
        if (targets == null || (list = sharedLabJson.floats(targets)) == null) {
            list = null;
        }
        linkedHashMap.put("targets", list);
        TractFit fit = state.getFit();
        if (fit != null) {
            Float valueOf = Float.valueOf(fit.getRmseHz());
            float floatValue = valueOf.floatValue();
            if (Float.isInfinite(floatValue) || Float.isNaN(floatValue)) {
                valueOf = null;
            }
            if (valueOf != null) {
                d = Double.valueOf(valueOf.floatValue());
            }
        }
        linkedHashMap.put("fitRmseHz", d);
        TractFit fit2 = state.getFit();
        linkedHashMap.put("fitClose", Boolean.valueOf(fit2 != null ? fit2.getCloseMatch() : false));
        List<Triple<String, String, float[]>> list2 = presets;
        ArrayList arrayList2 = new ArrayList(CollectionsKt.collectionSizeOrDefault(list2, 10));
        Iterator<T> it = list2.iterator();
        while (it.hasNext()) {
            Triple triple = (Triple) it.next();
            arrayList2.add(MapsKt.mapOf(TuplesKt.to("id", (String) triple.component1()), TuplesKt.to("label", (String) triple.component2()), TuplesKt.to("targets", INSTANCE.floats((float[]) triple.component3()))));
        }
        linkedHashMap.put("presets", arrayList2);
        return linkedHashMap;
    }

    public final String save(SharedLabState state) {
        Intrinsics.checkNotNullParameter(state, "state");
        String jSONObject = new JSONObject().put("schema", "shared-mri-lab-v1").put("modelId", state.getModel().getId()).put("coefficients", new JSONArray((Collection) floats(state.getCoefficients()))).put("f0", state.getF0()).put("master", state.getMaster()).put("sourceGains", new JSONArray((Collection) floats(state.getSourceGains()))).put("playing", false).toString();
        Intrinsics.checkNotNullExpressionValue(jSONObject, "toString(...)");
        return jSONObject;
    }

    public final boolean restore(SharedLabState state, String raw) {
        Intrinsics.checkNotNullParameter(state, "state");
        Intrinsics.checkNotNullParameter(raw, "raw");
        try {
            if (raw.length() > 8192) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            JSONObject jSONObject = new JSONObject(raw);
            if (!Intrinsics.areEqual(jSONObject.getString("schema"), "shared-mri-lab-v1") || !Intrinsics.areEqual(jSONObject.getString("modelId"), state.getModel().getId())) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            if (!(jSONObject.get("f0") instanceof Number) || !(jSONObject.get("master") instanceof Number)) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            state.restore(restore$array(jSONObject, "coefficients"), (float) jSONObject.getDouble("f0"), (float) jSONObject.getDouble("master"), restore$array(jSONObject, "sourceGains"));
            return true;
        } catch (Exception unused) {
            return false;
        }
    }

    private static final float[] restore$array(JSONObject jSONObject, String str) {
        JSONArray jSONArray = jSONObject.getJSONArray(str);
        int length = jSONArray.length();
        float[] fArr = new float[length];
        for (int i = 0; i < length; i++) {
            if (!(jSONArray.get(i) instanceof Number)) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            fArr[i] = (float) jSONArray.getDouble(i);
        }
        return fArr;
    }

    private final List<Double> floats(float[] a) {
        ArrayList arrayList = new ArrayList(a.length);
        for (float f : a) {
            arrayList.add(Double.valueOf(f));
        }
        return arrayList;
    }
}

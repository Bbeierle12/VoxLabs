package org.vocaltract.pixel;

import java.util.Iterator;
import java.util.Map;
import java.util.function.BiConsumer;
import kotlin.Metadata;
import kotlin.Unit;
import kotlin.collections.MapsKt;
import kotlin.jvm.functions.Function2;
import kotlin.jvm.internal.Intrinsics;
import org.json.JSONArray;
import org.json.JSONObject;

/* compiled from: DiagnosticStore.kt */
@Metadata(d1 = {"\u0000&\n\u0000\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\u0018\u0002\u001a\f\u0010\u0000\u001a\u00020\u0001*\u00020\u0002H\u0002\u001a\u0010\u0010\u0003\u001a\u00020\u00022\u0006\u0010\u0004\u001a\u00020\u0001H\u0002\u001a\f\u0010\u0000\u001a\u00020\u0001*\u00020\u0005H\u0002\u001a\f\u0010\u0000\u001a\u00020\u0001*\u00020\u0006H\u0002\u001a\f\u0010\u0000\u001a\u00020\u0001*\u00020\u0007H\u0002\u001a\f\u0010\u0000\u001a\u00020\u0001*\u00020\bH\u0002\u001a\f\u0010\u0000\u001a\u00020\u0001*\u00020\tH\u0002\u001a\f\u0010\u0000\u001a\u00020\u0001*\u00020\nH\u0002"}, d2 = {"toJson", "Lorg/json/JSONObject;", "Lorg/vocaltract/pixel/DiagnosticEvent;", "eventFromJson", "value", "Lorg/vocaltract/pixel/DiagnosticMetrics;", "Lorg/vocaltract/pixel/ArticulatorPosterior;", "Lorg/vocaltract/pixel/SingerCalibrationReport;", "Lorg/vocaltract/pixel/DiagnosticAssessment;", "Lorg/vocaltract/pixel/RefinementRecord;", "Lorg/vocaltract/pixel/SelfTestResult;"}, k = 2, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class DiagnosticStoreKt {
    /* JADX INFO: Access modifiers changed from: private */
    public static final JSONObject toJson(DiagnosticEvent diagnosticEvent) {
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("timestamp_epoch_ms", diagnosticEvent.getTimestampMillis());
        jSONObject.put("session_id", diagnosticEvent.getSessionId());
        jSONObject.put("category", diagnosticEvent.getCategory());
        jSONObject.put("code", diagnosticEvent.getCode());
        jSONObject.put("severity", diagnosticEvent.getSeverity().name());
        jSONObject.put("message", diagnosticEvent.getMessage());
        JSONObject jSONObject2 = new JSONObject();
        Map<String, String> evidence = diagnosticEvent.getEvidence();
        final DiagnosticStoreKt$toJson$1$1$1 diagnosticStoreKt$toJson$1$1$1 = new DiagnosticStoreKt$toJson$1$1$1(jSONObject2);
        evidence.forEach(new BiConsumer() { // from class: org.vocaltract.pixel.DiagnosticStoreKt$$ExternalSyntheticLambda0
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.util.function.BiConsumer
            public final void accept(Object obj, Object obj2) {
                Function2.this.invoke(obj, obj2);
            }
        });
        Unit unit = Unit.INSTANCE;
        jSONObject.put("evidence", jSONObject2);
        return jSONObject;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final DiagnosticEvent eventFromJson(JSONObject jSONObject) {
        JSONObject optJSONObject = jSONObject.optJSONObject("evidence");
        if (optJSONObject == null) {
            optJSONObject = new JSONObject();
        }
        Map createMapBuilder = MapsKt.createMapBuilder();
        Iterator<String> keys = optJSONObject.keys();
        while (keys.hasNext()) {
            String next = keys.next();
            createMapBuilder.put(next, optJSONObject.optString(next));
        }
        Map build = MapsKt.build(createMapBuilder);
        long j = jSONObject.getLong("timestamp_epoch_ms");
        String string = jSONObject.getString("session_id");
        Intrinsics.checkNotNullExpressionValue(string, "getString(...)");
        String string2 = jSONObject.getString("category");
        Intrinsics.checkNotNullExpressionValue(string2, "getString(...)");
        String string3 = jSONObject.getString("code");
        Intrinsics.checkNotNullExpressionValue(string3, "getString(...)");
        String string4 = jSONObject.getString("severity");
        Intrinsics.checkNotNullExpressionValue(string4, "getString(...)");
        DiagnosticSeverity valueOf = DiagnosticSeverity.valueOf(string4);
        String string5 = jSONObject.getString("message");
        Intrinsics.checkNotNullExpressionValue(string5, "getString(...)");
        return new DiagnosticEvent(j, string, string2, string3, valueOf, string5, build);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final JSONObject toJson(DiagnosticMetrics diagnosticMetrics) {
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("sequence", diagnosticMetrics.getSequence());
        jSONObject.put("source", diagnosticMetrics.getSource());
        jSONObject.put("processing_ms", diagnosticMetrics.getProcessingMs());
        jSONObject.put("frame_budget_ms", diagnosticMetrics.getFrameBudgetMs());
        jSONObject.put("dropped_frames", diagnosticMetrics.getDroppedFrames());
        jSONObject.put("voiced", diagnosticMetrics.getVoiced());
        jSONObject.put("f0_hz", diagnosticMetrics.getF0Hz() != null ? Double.valueOf(r1.floatValue()) : JSONObject.NULL);
        jSONObject.put("f0_confidence", diagnosticMetrics.getF0Confidence());
        jSONObject.put("raw_f0_hz", diagnosticMetrics.getRawF0Hz() != null ? Double.valueOf(r1.floatValue()) : JSONObject.NULL);
        jSONObject.put("pitch_decision", diagnosticMetrics.getPitchDecision());
        jSONObject.put("pitch_rejected", diagnosticMetrics.getPitchRejected());
        jSONObject.put("harmonicity", diagnosticMetrics.getHarmonicity());
        jSONObject.put("snr_db", diagnosticMetrics.getSnrDb());
        jSONObject.put("noise_floor_db", diagnosticMetrics.getNoiseFloorDb());
        jSONObject.put("noise_state", diagnosticMetrics.getNoiseState());
        jSONObject.put("noise_confidence", diagnosticMetrics.getNoiseConfidence());
        jSONObject.put("background_changed", diagnosticMetrics.getBackgroundChanged());
        JSONArray jSONArray = new JSONArray();
        Iterator<T> it = diagnosticMetrics.getNoiseBandsDb().iterator();
        while (it.hasNext()) {
            jSONArray.put(((Number) it.next()).floatValue());
        }
        Unit unit = Unit.INSTANCE;
        jSONObject.put("noise_bands_db", jSONArray);
        jSONObject.put("mean_formant_std_hz", diagnosticMetrics.getMeanFormantStdHz());
        JSONArray jSONArray2 = new JSONArray();
        Iterator<T> it2 = diagnosticMetrics.getFormantCandidatesHz().iterator();
        while (it2.hasNext()) {
            jSONArray2.put(((Number) it2.next()).floatValue());
        }
        Unit unit2 = Unit.INSTANCE;
        jSONObject.put("formant_candidates_hz", jSONArray2);
        jSONObject.put("tract_confidence", diagnosticMetrics.getTractConfidence());
        jSONObject.put("relative_area_std", diagnosticMetrics.getRelativeAreaStd());
        jSONObject.put("posterior_abstained", diagnosticMetrics.getPosteriorAbstained());
        jSONObject.put("abstention_reason", diagnosticMetrics.getAbstentionReason());
        jSONObject.put("analysis_window_ms", diagnosticMetrics.getAnalysisWindowMs());
        jSONObject.put("analysis_hop_ms", diagnosticMetrics.getAnalysisHopMs());
        jSONObject.put("articulators", toJson(diagnosticMetrics.getArticulators()));
        jSONObject.put("renderer_mode", diagnosticMetrics.getRendererMode());
        jSONObject.put("microphone_granted", diagnosticMetrics.getMicrophoneGranted());
        jSONObject.put("synthesizer_active", diagnosticMetrics.getSynthesizerActive());
        return jSONObject;
    }

    private static final JSONObject toJson(ArticulatorPosterior articulatorPosterior) {
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("jaw_opening", articulatorPosterior.getJawOpening());
        jSONObject.put("lip_aperture", articulatorPosterior.getLipAperture());
        jSONObject.put("tongue_front_back", articulatorPosterior.getTongueFrontBack());
        jSONObject.put("tongue_dorsum", articulatorPosterior.getTongueDorsum());
        jSONObject.put("tongue_root", articulatorPosterior.getTongueRoot());
        jSONObject.put("pharynx_width", articulatorPosterior.getPharynxWidth());
        jSONObject.put("epilarynx_width", articulatorPosterior.getEpilarynxWidth());
        jSONObject.put("lip_protrusion", articulatorPosterior.getLipProtrusion() != null ? Double.valueOf(r1.floatValue()) : JSONObject.NULL);
        jSONObject.put("larynx_height", articulatorPosterior.getLarynxHeight() != null ? Double.valueOf(r1.floatValue()) : JSONObject.NULL);
        jSONObject.put("velum_opening", articulatorPosterior.getVelumOpening() != null ? Double.valueOf(r1.floatValue()) : JSONObject.NULL);
        jSONObject.put("nasal_coupling", articulatorPosterior.getNasalCoupling() != null ? Double.valueOf(r1.floatValue()) : JSONObject.NULL);
        jSONObject.put("interpretation", articulatorPosterior.getInterpretation());
        return jSONObject;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final JSONObject toJson(SingerCalibrationReport singerCalibrationReport) {
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("schema_version", singerCalibrationReport.getSchemaVersion());
        jSONObject.put("started_at_epoch_ms", singerCalibrationReport.getStartedAtMillis());
        jSONObject.put("completed_at_epoch_ms", singerCalibrationReport.getCompletedAtMillis());
        jSONObject.put("raw_audio_stored", singerCalibrationReport.getRawAudioStored());
        jSONObject.put("automatic_model_mutation", singerCalibrationReport.getAutomaticModelMutation());
        JSONArray jSONArray = new JSONArray();
        for (CalibrationStepSummary calibrationStepSummary : singerCalibrationReport.getSteps()) {
            JSONObject jSONObject2 = new JSONObject();
            jSONObject2.put("id", calibrationStepSummary.getId());
            jSONObject2.put("frames", calibrationStepSummary.getFrames());
            jSONObject2.put("voiced_frames", calibrationStepSummary.getVoicedFrames());
            jSONObject2.put("mean_f0_hz", calibrationStepSummary.getMeanF0Hz() != null ? Double.valueOf(r4.floatValue()) : JSONObject.NULL);
            JSONArray jSONArray2 = new JSONArray();
            Iterator<T> it = calibrationStepSummary.getMeanFormantsHz().iterator();
            while (it.hasNext()) {
                jSONArray2.put(((Number) it.next()).floatValue());
            }
            Unit unit = Unit.INSTANCE;
            jSONObject2.put("mean_formants_hz", jSONArray2);
            jSONObject2.put("mean_snr_db", calibrationStepSummary.getMeanSnrDb());
            jSONObject2.put("mean_tract_confidence", calibrationStepSummary.getMeanTractConfidence());
            jSONObject2.put("abstained_frames", calibrationStepSummary.getAbstainedFrames());
            jSONArray.put(jSONObject2);
        }
        Unit unit2 = Unit.INSTANCE;
        jSONObject.put("steps", jSONArray);
        return jSONObject;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final JSONObject toJson(DiagnosticAssessment diagnosticAssessment) {
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("provider_id", diagnosticAssessment.getProviderId());
        jSONObject.put("generated_at_epoch_ms", diagnosticAssessment.getGeneratedAtMillis());
        jSONObject.put("health_score", diagnosticAssessment.getHealthScore());
        jSONObject.put("summary", diagnosticAssessment.getSummary());
        jSONObject.put("automatic_model_mutation_allowed", diagnosticAssessment.getAutomaticModelMutationAllowed());
        JSONArray jSONArray = new JSONArray();
        for (DiagnosticFinding diagnosticFinding : diagnosticAssessment.getFindings()) {
            JSONObject jSONObject2 = new JSONObject();
            jSONObject2.put("code", diagnosticFinding.getCode());
            jSONObject2.put("severity", diagnosticFinding.getSeverity().name());
            jSONObject2.put("title", diagnosticFinding.getTitle());
            jSONObject2.put("evidence", diagnosticFinding.getEvidence());
            jSONObject2.put("recommended_action", diagnosticFinding.getRecommendedAction());
            jSONObject2.put("confidence", diagnosticFinding.getConfidence());
            jSONArray.put(jSONObject2);
        }
        Unit unit = Unit.INSTANCE;
        jSONObject.put("findings", jSONArray);
        return jSONObject;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final JSONObject toJson(RefinementRecord refinementRecord) {
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("schema_version", "vocaltract3d.refinement/1.0");
        jSONObject.put("record_id", refinementRecord.getRecordId());
        jSONObject.put("timestamp_epoch_ms", refinementRecord.getTimestampMillis());
        jSONObject.put("session_id", refinementRecord.getSessionId());
        jSONObject.put("status", refinementRecord.getStatus());
        jSONObject.put("observed", toJson(refinementRecord.getObserved()));
        JSONObject jSONObject2 = new JSONObject();
        jSONObject2.put("expected_f0_hz", refinementRecord.getCorrection().getExpectedF0Hz() != null ? Double.valueOf(r2.floatValue()) : JSONObject.NULL);
        Object expectedVowel = refinementRecord.getCorrection().getExpectedVowel();
        if (expectedVowel == null) {
            expectedVowel = JSONObject.NULL;
        }
        jSONObject2.put("expected_vowel", expectedVowel);
        JSONArray jSONArray = new JSONArray();
        Iterator<T> it = refinementRecord.getCorrection().getExpectedResonancesHz().iterator();
        while (it.hasNext()) {
            jSONArray.put(((Number) it.next()).floatValue());
        }
        Unit unit = Unit.INSTANCE;
        jSONObject2.put("expected_resonances_hz", jSONArray);
        jSONObject2.put("notes", refinementRecord.getCorrection().getNotes());
        jSONObject2.put("approved_for_future_training", refinementRecord.getCorrection().getApprovedForFutureTraining());
        Unit unit2 = Unit.INSTANCE;
        jSONObject.put("correction", jSONObject2);
        return jSONObject;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final JSONObject toJson(SelfTestResult selfTestResult) {
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("code", selfTestResult.getCode());
        jSONObject.put("passed", selfTestResult.getPassed());
        jSONObject.put("message", selfTestResult.getMessage());
        return jSONObject;
    }
}

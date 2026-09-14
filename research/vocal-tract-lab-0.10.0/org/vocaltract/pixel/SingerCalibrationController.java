package org.vocaltract.pixel;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Iterator;
import java.util.List;
import kotlin.Metadata;
import kotlin.Unit;
import kotlin.collections.CollectionsKt;
import kotlin.jvm.functions.Function0;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: SingerCalibration.kt */
@Metadata(d1 = {"\u0000`\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0018\u0002\n\u0002\u0010\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0010 \n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0010\t\n\u0002\b\u0002\n\u0002\u0010\b\n\u0000\n\u0002\u0010!\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0002\b\u0007\n\u0002\u0010\u000e\n\u0000\u0018\u00002\u00020\u0001B)\u0012\f\u0010\u0002\u001a\b\u0012\u0004\u0012\u00020\u00040\u0003\u0012\u0012\u0010\u0005\u001a\u000e\u0012\u0004\u0012\u00020\u0007\u0012\u0004\u0012\u00020\u00040\u0006¢\u0006\u0004\b\b\u0010\tJ\u0010\u0010\u001c\u001a\u00020\u001d2\b\b\u0002\u0010\u001e\u001a\u00020\u0010J\u0006\u0010\u001f\u001a\u00020\u001dJ\u0018\u0010 \u001a\u00020\u001d2\u0006\u0010!\u001a\u00020\u00182\b\b\u0002\u0010\u001e\u001a\u00020\u0010J\u0010\u0010\"\u001a\u00020\u001d2\b\b\u0002\u0010\u001e\u001a\u00020\u0010J\u001e\u0010#\u001a\u00020\u00162\u0006\u0010$\u001a\u00020%2\f\u0010&\u001a\b\u0012\u0004\u0012\u00020\u00180\u000bH\u0002R\u0014\u0010\u0002\u001a\b\u0012\u0004\u0012\u00020\u00040\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u001a\u0010\u0005\u001a\u000e\u0012\u0004\u0012\u00020\u0007\u0012\u0004\u0012\u00020\u00040\u0006X\u0082\u0004¢\u0006\u0002\n\u0000R\u0017\u0010\n\u001a\b\u0012\u0004\u0012\u00020\f0\u000b¢\u0006\b\n\u0000\u001a\u0004\b\r\u0010\u000eR\u000e\u0010\u000f\u001a\u00020\u0010X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0011\u001a\u00020\u0010X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0012\u001a\u00020\u0013X\u0082\u000e¢\u0006\u0002\n\u0000R\u0014\u0010\u0014\u001a\b\u0012\u0004\u0012\u00020\u00160\u0015X\u0082\u0004¢\u0006\u0002\n\u0000R\u0014\u0010\u0017\u001a\b\u0012\u0004\u0012\u00020\u00180\u0015X\u0082\u0004¢\u0006\u0002\n\u0000R\u0011\u0010\u0019\u001a\u00020\u001a8F¢\u0006\u0006\u001a\u0004\b\u0019\u0010\u001b"}, d2 = {"Lorg/vocaltract/pixel/SingerCalibrationController;", "", "onNoiseLearningRequested", "Lkotlin/Function0;", "", "onCompleted", "Lkotlin/Function1;", "Lorg/vocaltract/pixel/SingerCalibrationReport;", "<init>", "(Lkotlin/jvm/functions/Function0;Lkotlin/jvm/functions/Function1;)V", "steps", "", "Lorg/vocaltract/pixel/CalibrationStep;", "getSteps", "()Ljava/util/List;", "startedAtMillis", "", "stepStartedAtMillis", "index", "", "summaries", "", "Lorg/vocaltract/pixel/CalibrationStepSummary;", "frames", "Lorg/vocaltract/pixel/VocalAcousticsState;", "isActive", "", "()Z", "start", "Lorg/vocaltract/pixel/CalibrationProgress;", "nowMillis", "cancel", "accept", "state", "progress", "summarize", "id", "", "values"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class SingerCalibrationController {
    private final List<VocalAcousticsState> frames;
    private int index;
    private final Function1<SingerCalibrationReport, Unit> onCompleted;
    private final Function0<Unit> onNoiseLearningRequested;
    private long startedAtMillis;
    private long stepStartedAtMillis;
    private final List<CalibrationStep> steps;
    private final List<CalibrationStepSummary> summaries;

    /* JADX DEBUG: Multi-variable search result rejected for r11v0, resolved type: kotlin.jvm.functions.Function1<? super org.vocaltract.pixel.SingerCalibrationReport, kotlin.Unit> */
    /* JADX WARN: Multi-variable type inference failed */
    public SingerCalibrationController(Function0<Unit> onNoiseLearningRequested, Function1<? super SingerCalibrationReport, Unit> onCompleted) {
        Intrinsics.checkNotNullParameter(onNoiseLearningRequested, "onNoiseLearningRequested");
        Intrinsics.checkNotNullParameter(onCompleted, "onCompleted");
        this.onNoiseLearningRequested = onNoiseLearningRequested;
        this.onCompleted = onCompleted;
        this.steps = CollectionsKt.listOf((Object[]) new CalibrationStep[]{new CalibrationStep("background", "Stay quiet while the normal room background (including the fan) is learned", 4000L, true), new CalibrationStep("vowel_a", "Sustain /a/ at a comfortable pitch", 3000L, false, 8, null), new CalibrationStep("vowel_e", "Sustain /e/ at a comfortable pitch", 3000L, false, 8, null), new CalibrationStep("vowel_i", "Sustain /i/ at a comfortable pitch", 3000L, false, 8, null), new CalibrationStep("vowel_o", "Sustain /o/ at a comfortable pitch", 3000L, false, 8, null), new CalibrationStep("vowel_u", "Sustain /u/ at a comfortable pitch", 3000L, false, 8, null), new CalibrationStep("glide", "Glide smoothly from low to high and back", 6000L, false, 8, null)});
        this.index = -1;
        this.summaries = new ArrayList();
        this.frames = new ArrayList();
    }

    public final List<CalibrationStep> getSteps() {
        return this.steps;
    }

    public final boolean isActive() {
        int size = this.steps.size();
        int i = this.index;
        return i >= 0 && i < size;
    }

    public static /* synthetic */ CalibrationProgress start$default(SingerCalibrationController singerCalibrationController, long j, int i, Object obj) {
        if ((i & 1) != 0) {
            j = System.currentTimeMillis();
        }
        return singerCalibrationController.start(j);
    }

    public final CalibrationProgress start(long nowMillis) {
        this.startedAtMillis = nowMillis;
        this.stepStartedAtMillis = nowMillis;
        this.index = 0;
        this.summaries.clear();
        this.frames.clear();
        this.onNoiseLearningRequested.invoke();
        return progress(nowMillis);
    }

    public final CalibrationProgress cancel() {
        this.index = -1;
        this.frames.clear();
        this.summaries.clear();
        return new CalibrationProgress(false, 0, this.steps.size(), "Calibration cancelled", 0.0f, null, 32, null);
    }

    public static /* synthetic */ CalibrationProgress accept$default(SingerCalibrationController singerCalibrationController, VocalAcousticsState vocalAcousticsState, long j, int i, Object obj) {
        if ((i & 2) != 0) {
            j = System.currentTimeMillis();
        }
        return singerCalibrationController.accept(vocalAcousticsState, j);
    }

    public final CalibrationProgress accept(VocalAcousticsState state, long nowMillis) {
        Intrinsics.checkNotNullParameter(state, "state");
        if (!isActive()) {
            return progress(nowMillis);
        }
        this.frames.add(state);
        CalibrationStep calibrationStep = this.steps.get(this.index);
        if (nowMillis - this.stepStartedAtMillis < calibrationStep.getDurationMillis()) {
            return progress(nowMillis);
        }
        this.summaries.add(summarize(calibrationStep.getId(), this.frames));
        this.frames.clear();
        int i = this.index + 1;
        this.index = i;
        if (i >= this.steps.size()) {
            SingerCalibrationReport singerCalibrationReport = new SingerCalibrationReport(null, this.startedAtMillis, nowMillis, CollectionsKt.toList(this.summaries), false, false, 49, null);
            this.index = -1;
            this.onCompleted.invoke(singerCalibrationReport);
            return new CalibrationProgress(false, this.steps.size(), this.steps.size(), "Calibration complete", 1.0f, singerCalibrationReport);
        }
        this.stepStartedAtMillis = nowMillis;
        if (this.steps.get(this.index).getForceNoiseLearning()) {
            this.onNoiseLearningRequested.invoke();
        }
        return progress(nowMillis);
    }

    public static /* synthetic */ CalibrationProgress progress$default(SingerCalibrationController singerCalibrationController, long j, int i, Object obj) {
        if ((i & 1) != 0) {
            j = System.currentTimeMillis();
        }
        return singerCalibrationController.progress(j);
    }

    public final CalibrationProgress progress(long nowMillis) {
        if (!isActive()) {
            return new CalibrationProgress(false, 0, this.steps.size(), "Calibration idle", 0.0f, null, 32, null);
        }
        return new CalibrationProgress(true, this.index, this.steps.size(), this.steps.get(this.index).getPrompt(), RangesKt.coerceIn((nowMillis - this.stepStartedAtMillis) / r0.getDurationMillis(), 0.0f, 1.0f), null, 32, null);
    }

    private final CalibrationStepSummary summarize(String id, List<VocalAcousticsState> values) {
        Integer valueOf;
        float f;
        float f2;
        List<VocalAcousticsState> list = values;
        ArrayList arrayList = new ArrayList();
        Iterator<T> it = list.iterator();
        while (it.hasNext()) {
            Float f0Hz = ((VocalAcousticsState) it.next()).getAcoustic().getF0Hz();
            if (f0Hz != null) {
                arrayList.add(f0Hz);
            }
        }
        ArrayList arrayList2 = arrayList;
        Iterator<T> it2 = list.iterator();
        if (it2.hasNext()) {
            valueOf = Integer.valueOf(((VocalAcousticsState) it2.next()).getAcoustic().getFormantsHz().size());
            while (it2.hasNext()) {
                Integer valueOf2 = Integer.valueOf(((VocalAcousticsState) it2.next()).getAcoustic().getFormantsHz().size());
                if (valueOf.compareTo(valueOf2) < 0) {
                    valueOf = valueOf2;
                }
            }
        } else {
            valueOf = null;
        }
        Integer num = valueOf;
        int i = 0;
        int intValue = num != null ? num.intValue() : 0;
        ArrayList arrayList3 = new ArrayList(intValue);
        int i2 = 0;
        while (true) {
            f = 0.0f;
            if (i2 >= intValue) {
                break;
            }
            ArrayList arrayList4 = new ArrayList();
            Iterator<T> it3 = list.iterator();
            while (it3.hasNext()) {
                Float f3 = (Float) CollectionsKt.getOrNull(((VocalAcousticsState) it3.next()).getAcoustic().getFormantsHz(), i2);
                if (f3 != null) {
                    arrayList4.add(f3);
                }
            }
            ArrayList arrayList5 = arrayList4;
            if (!(!arrayList5.isEmpty())) {
                arrayList5 = null;
            }
            if (arrayList5 != null) {
                f = (float) CollectionsKt.averageOfFloat(arrayList5);
            }
            arrayList3.add(Float.valueOf(f));
            i2++;
        }
        ArrayList arrayList6 = arrayList3;
        int size = values.size();
        int size2 = arrayList2.size();
        if (!(!arrayList2.isEmpty())) {
            arrayList2 = null;
        }
        Float valueOf3 = arrayList2 != null ? Float.valueOf((float) CollectionsKt.averageOfFloat(arrayList2)) : null;
        List<VocalAcousticsState> list2 = values;
        List<VocalAcousticsState> list3 = list2.isEmpty() ^ true ? values : null;
        if (list3 != null) {
            List<VocalAcousticsState> list4 = list3;
            ArrayList arrayList7 = new ArrayList(CollectionsKt.collectionSizeOrDefault(list4, 10));
            Iterator<T> it4 = list4.iterator();
            while (it4.hasNext()) {
                arrayList7.add(Float.valueOf(((VocalAcousticsState) it4.next()).getAcoustic().getSnrDb()));
            }
            f2 = (float) CollectionsKt.averageOfFloat(arrayList7);
        } else {
            f2 = 0.0f;
        }
        List<VocalAcousticsState> list5 = list2.isEmpty() ^ true ? values : null;
        if (list5 != null) {
            List<VocalAcousticsState> list6 = list5;
            ArrayList arrayList8 = new ArrayList(CollectionsKt.collectionSizeOrDefault(list6, 10));
            Iterator<T> it5 = list6.iterator();
            while (it5.hasNext()) {
                arrayList8.add(Float.valueOf(((VocalAcousticsState) it5.next()).getTract().getConfidence()));
            }
            f = (float) CollectionsKt.averageOfFloat(arrayList8);
        }
        float f4 = f;
        if (!(list instanceof Collection) || !list.isEmpty()) {
            Iterator<T> it6 = list.iterator();
            while (it6.hasNext()) {
                if (((VocalAcousticsState) it6.next()).getTract().getAbstained() && (i = i + 1) < 0) {
                    CollectionsKt.throwCountOverflow();
                }
            }
        }
        return new CalibrationStepSummary(id, size, size2, valueOf3, arrayList6, f2, f4, i);
    }
}

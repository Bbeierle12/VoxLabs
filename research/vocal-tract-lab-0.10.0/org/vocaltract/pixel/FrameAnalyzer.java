package org.vocaltract.pixel;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.Comparator;
import java.util.Iterator;
import java.util.List;
import kotlin.Metadata;
import kotlin.Pair;
import kotlin.collections.ArraysKt;
import kotlin.collections.CollectionsKt;
import kotlin.collections.IntIterator;
import kotlin.comparisons.ComparisonsKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.math.MathKt;
import kotlin.ranges.IntRange;
import kotlin.ranges.RangesKt;

/* compiled from: RealtimeDspPipeline.kt */
@Metadata(d1 = {"\u0000n\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\b\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0014\n\u0002\b\u0006\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0004\n\u0002\u0010\u0007\n\u0000\n\u0002\u0010\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\t\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0002\b\b\n\u0002\u0010 \n\u0002\b\u0007\n\u0002\u0018\u0002\b\u0000\u0018\u00002\u00020\u0001B-\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\n\b\u0002\u0010\u0006\u001a\u0004\u0018\u00010\u0007\u0012\b\b\u0002\u0010\b\u001a\u00020\t¢\u0006\u0004\b\n\u0010\u000bJ\u000e\u0010\u001a\u001a\u00020\u001b2\u0006\u0010\u001c\u001a\u00020\u0005J\u001e\u0010\u001d\u001a\u00020\u001e2\u0006\u0010\u001f\u001a\u00020 2\u0006\u0010!\u001a\u00020\"2\u0006\u0010#\u001a\u00020\"J\u001e\u0010$\u001a\u0010\u0012\u0006\u0012\u0004\u0018\u00010\u0019\u0012\u0004\u0012\u00020\u00190%2\u0006\u0010&\u001a\u00020\tH\u0002J\u0010\u0010'\u001a\u00020\t2\u0006\u0010&\u001a\u00020\tH\u0002J\u0010\u0010(\u001a\u00020\t2\u0006\u0010)\u001a\u00020\tH\u0002J\u001f\u0010*\u001a\u00020\u00192\b\u0010+\u001a\u0004\u0018\u00010\u00192\u0006\u0010)\u001a\u00020\tH\u0002¢\u0006\u0002\u0010,J0\u0010-\u001a\u001a\u0012\n\u0012\b\u0012\u0004\u0012\u00020\u00190.\u0012\n\u0012\b\u0012\u0004\u0012\u00020\u00190.0%2\u0006\u0010/\u001a\u00020\t2\u0006\u0010+\u001a\u00020\u0019H\u0002J;\u00100\u001a\b\u0012\u0004\u0012\u00020\u00190.2\f\u00101\u001a\b\u0012\u0004\u0012\u00020\u00190.2\b\u0010+\u001a\u0004\u0018\u00010\u00192\u0006\u00102\u001a\u00020\u00192\u0006\u00103\u001a\u00020\u0019H\u0002¢\u0006\u0002\u00104J\u001e\u00105\u001a\b\u0012\u0004\u0012\u0002060.2\u0006\u0010+\u001a\u00020\u00192\u0006\u0010/\u001a\u00020\tH\u0002R\u000e\u0010\u0002\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0004\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u0010\u0010\u0006\u001a\u0004\u0018\u00010\u0007X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\f\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\r\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u000e\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u000f\u001a\u00020\u0010X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0011\u001a\u00020\u0012X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0013\u001a\u00020\u0014X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0015\u001a\u00020\u0005X\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010\u0016\u001a\u0004\u0018\u00010\tX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0017\u001a\u00020\tX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0018\u001a\u00020\u0019X\u0082\u000e¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/FrameAnalyzer;", "", "asset", "Lorg/vocaltract/pixel/ReducedModelAsset;", "maxHarmonics", "", "sharedModel", "Lorg/vocaltract/pixel/SharedTractModel;", "seed", "", "<init>", "(Lorg/vocaltract/pixel/ReducedModelAsset;ILorg/vocaltract/pixel/SharedTractModel;[F)V", "sampleRate", "frameSize", "hann", "noiseTracker", "Lorg/vocaltract/pixel/StationaryNoiseTracker;", "pitchGate", "Lorg/vocaltract/pixel/PitchContinuityGate;", "temporalFilter", "Lorg/vocaltract/pixel/TemporalAtlasFilter;", "forcedNoiseFrames", "trackedFormants", "fitCoordinates", "fitRmseHz", "", "requestNoiseLearning", "", "frames", "analyze", "Lorg/vocaltract/pixel/VocalAcousticsState;", "frame", "Lorg/vocaltract/pixel/PcmFrame;", "sequence", "", "droppedFrames", "estimateF0", "Lkotlin/Pair;", "samples", "spectrumPower", "relativeDb", "power", "harmonicity", "f0", "(Ljava/lang/Float;[F)F", "estimateFormants", "", "spectrumDb", "trackFormants", "observed", "f0Confidence", "snrDb", "(Ljava/util/List;Ljava/lang/Float;FF)Ljava/util/List;", "buildHarmonics", "Lorg/vocaltract/pixel/HarmonicEstimate;"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class FrameAnalyzer {
    private final ReducedModelAsset asset;
    private float[] fitCoordinates;
    private float fitRmseHz;
    private volatile int forcedNoiseFrames;
    private final int frameSize;
    private final float[] hann;
    private final int maxHarmonics;
    private final StationaryNoiseTracker noiseTracker;
    private final PitchContinuityGate pitchGate;
    private final int sampleRate;
    private final SharedTractModel sharedModel;
    private final TemporalAtlasFilter temporalFilter;
    private float[] trackedFormants;

    public FrameAnalyzer(ReducedModelAsset asset, int i, SharedTractModel sharedTractModel, float[] seed) {
        Intrinsics.checkNotNullParameter(asset, "asset");
        Intrinsics.checkNotNullParameter(seed, "seed");
        this.asset = asset;
        this.maxHarmonics = i;
        this.sharedModel = sharedTractModel;
        this.sampleRate = asset.getSampleRateHz();
        int frameSize = asset.getFrameSize();
        this.frameSize = frameSize;
        float[] fArr = new float[frameSize];
        int i2 = 0;
        for (int i3 = 0; i3 < frameSize; i3++) {
            fArr[i3] = (float) (0.5d - (Math.cos((i3 * 6.283185307179586d) / (this.frameSize - 1)) * 0.5d));
        }
        this.hann = fArr;
        this.noiseTracker = new StationaryNoiseTracker((this.frameSize / 2) + 1, i2, 2, null);
        this.pitchGate = new PitchContinuityGate();
        this.temporalFilter = new TemporalAtlasFilter(this.asset, seed);
        float[] copyOf = Arrays.copyOf(seed, seed.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        this.fitCoordinates = copyOf;
        this.fitRmseHz = Float.POSITIVE_INFINITY;
    }

    public /* synthetic */ FrameAnalyzer(ReducedModelAsset reducedModelAsset, int i, SharedTractModel sharedTractModel, float[] fArr, int i2, DefaultConstructorMarker defaultConstructorMarker) {
        this(reducedModelAsset, i, (i2 & 4) != 0 ? null : sharedTractModel, (i2 & 8) != 0 ? new float[reducedModelAsset.getAreaModes().length] : fArr);
    }

    public final void requestNoiseLearning(int frames) {
        this.forcedNoiseFrames = Math.max(this.forcedNoiseFrames, RangesKt.coerceIn(frames, 1, 300));
    }

    /* JADX WARN: Code restructure failed: missing block: B:38:0x0115, code lost:
    
        if (java.lang.Float.isNaN(r10) == false) goto L76;
     */
    /* JADX WARN: Code restructure failed: missing block: B:54:0x024c, code lost:
    
        if (r50.fitRmseHz <= 250.0f) goto L95;
     */
    /* JADX WARN: Removed duplicated region for block: B:58:0x0256  */
    /* JADX WARN: Removed duplicated region for block: B:68:0x0274  */
    /* JADX WARN: Removed duplicated region for block: B:70:0x0277  */
    /* JADX WARN: Removed duplicated region for block: B:78:0x02c7  */
    /* JADX WARN: Removed duplicated region for block: B:88:0x02f6  */
    /* JADX WARN: Removed duplicated region for block: B:91:0x032e  */
    /* JADX WARN: Removed duplicated region for block: B:95:0x0331  */
    /* JADX WARN: Removed duplicated region for block: B:96:0x0312  */
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    public final VocalAcousticsState analyze(PcmFrame frame, long sequence, long droppedFrames) {
        Pair<List<Float>, List<Float>> pair;
        float f;
        Float f2;
        boolean z;
        float[] second;
        String str;
        float coerceIn;
        boolean z2;
        float f3;
        boolean z3;
        TemporalAtlasEstimate update;
        SharedTractModel sharedTractModel;
        Iterator<T> it;
        float f4;
        Float f5;
        int i;
        Intrinsics.checkNotNullParameter(frame, "frame");
        long nanoTime = System.nanoTime();
        int length = frame.getSamples().length;
        int i2 = this.frameSize;
        if (length != i2) {
            throw new IllegalArgumentException(("Expected " + i2 + " PCM samples, received " + frame.getSamples().length).toString());
        }
        float[] fArr = new float[i2];
        float f6 = 0.0f;
        for (int i3 = 0; i3 < frame.getSamples().length; i3++) {
            f6 += r6[i3] / 32768.0f;
        }
        float f7 = f6 / this.frameSize;
        double d = 0.0d;
        for (int i4 = 0; i4 < i2; i4++) {
            fArr[i4] = (frame.getSamples()[i4] / 32768.0f) - f7;
            d += r13 * r13;
        }
        float sqrt = (float) Math.sqrt(d / this.frameSize);
        Pair<Float, Float> estimateF0 = sqrt >= 0.004f ? estimateF0(fArr) : new Pair<>(null, Float.valueOf(0.0f));
        Float component1 = estimateF0.component1();
        float floatValue = estimateF0.component2().floatValue();
        float[] spectrumPower = spectrumPower(fArr);
        boolean z4 = this.forcedNoiseFrames > 0;
        if (z4) {
            this.forcedNoiseFrames--;
        }
        NoiseEstimate update2 = this.noiseTracker.update(spectrumPower, component1 != null && floatValue >= 0.45f, z4);
        float harmonicity = harmonicity(component1, update2.getCleanPower());
        PitchDecision update3 = this.pitchGate.update(component1, floatValue, harmonicity);
        Float acceptedF0Hz = update3.getAcceptedF0Hz();
        float[] relativeDb = relativeDb(update2.getCleanPower());
        if (acceptedF0Hz != null) {
            pair = estimateFormants(relativeDb, acceptedF0Hz.floatValue());
        } else {
            pair = new Pair<>(ArraysKt.toList(this.asset.getReferenceFormantsHz()), CollectionsKt.emptyList());
        }
        List<Float> component12 = pair.component1();
        List<Float> component2 = pair.component2();
        List<Float> trackFormants = trackFormants(component12, acceptedF0Hz, floatValue, update2.getSnrDb());
        List<HarmonicEstimate> emptyList = acceptedF0Hz == null ? CollectionsKt.emptyList() : buildHarmonics(acceptedF0Hz.floatValue(), relativeDb);
        if (this.sharedModel != null) {
            if (sequence % 4 != 0) {
                float f8 = this.fitRmseHz;
                if (!Float.isInfinite(f8)) {
                }
            }
            if (acceptedF0Hz != null && !z4 && floatValue >= 0.45f && component2.size() >= 3) {
                float[] floatArray = CollectionsKt.toFloatArray(CollectionsKt.take(trackFormants, 3));
                f2 = component1;
                if (floatArray.length == 3) {
                    int length2 = floatArray.length;
                    while (i < length2) {
                        float f9 = floatArray[i];
                        i = (!Float.isInfinite(f9) && !Float.isNaN(f9) && 150.0f <= f9 && f9 <= 4900.0f) ? i + 1 : 0;
                    }
                    f = sqrt;
                    Iterable intRange = new IntRange(1, 2);
                    if (!(intRange instanceof Collection) || !((Collection) intRange).isEmpty()) {
                        Iterator it2 = intRange.iterator();
                        while (it2.hasNext()) {
                            int nextInt = ((IntIterator) it2).nextInt();
                            z = true;
                            if (floatArray[nextInt] <= floatArray[nextInt - 1]) {
                                break;
                            }
                        }
                    }
                    z = true;
                    TractFit fit = this.sharedModel.fit(floatArray, this.fitCoordinates, 2);
                    this.fitCoordinates = fit.getCoefficients();
                    this.fitRmseHz = fit.getRmseHz();
                    float[] fArr2 = this.fitCoordinates;
                    second = Arrays.copyOf(fArr2, fArr2.length);
                    Intrinsics.checkNotNullExpressionValue(second, "copyOf(...)");
                }
                f = sqrt;
                z = true;
                float[] fArr22 = this.fitCoordinates;
                second = Arrays.copyOf(fArr22, fArr22.length);
                Intrinsics.checkNotNullExpressionValue(second, "copyOf(...)");
            }
            f = sqrt;
            f2 = component1;
            z = true;
            float[] fArr222 = this.fitCoordinates;
            second = Arrays.copyOf(fArr222, fArr222.length);
            Intrinsics.checkNotNullExpressionValue(second, "copyOf(...)");
        } else {
            f = sqrt;
            f2 = component1;
            z = true;
            second = this.asset.inferArea(trackFormants).getSecond();
        }
        float coerceIn2 = RangesKt.coerceIn((RangesKt.coerceIn(floatValue, 0.0f, 1.0f) * 0.42f) + (0.3f * harmonicity) + (RangesKt.coerceIn((update2.getSnrDb() + 5.0f) / 25.0f, 0.0f, 1.0f) * 0.28f), 0.0f, 1.0f);
        if (acceptedF0Hz == null) {
            str = "copyOf(...)";
            coerceIn = 0.0f;
        } else {
            str = "copyOf(...)";
            coerceIn = RangesKt.coerceIn((acceptedF0Hz.floatValue() - 420.0f) / 600.0f, 0.0f, 0.28f);
        }
        float coerceIn3 = RangesKt.coerceIn(((0.78f * coerceIn2) + 0.05f) - coerceIn, 0.03f, 0.82f);
        if (this.sharedModel != null) {
            if (component2.size() >= 3) {
                float f10 = this.fitRmseHz;
                if (!Float.isInfinite(f10)) {
                    if (!Float.isNaN(f10)) {
                    }
                }
            }
            z2 = z;
            TemporalAtlasFilter temporalAtlasFilter = this.temporalFilter;
            if (z2) {
                coerceIn3 = 0.0f;
            }
            if (acceptedF0Hz != null || z4) {
                f3 = harmonicity;
                z3 = false;
            } else {
                f3 = harmonicity;
                z3 = z;
            }
            update = temporalAtlasFilter.update(second, coerceIn3, z3);
            if (z2 && acceptedF0Hz != null && !z4) {
                update = TemporalAtlasEstimate.copy$default(update, null, 0.0f, false, component2.size() >= 3 ? "insufficient_formant_peaks" : "model_mismatch", 7, null);
            }
            sharedTractModel = this.sharedModel;
            if (sharedTractModel != null || (r2 = sharedTractModel.area(update.getCoefficients())) == null) {
                float[] areaFromCoefficients = this.asset.areaFromCoefficients(update.getCoefficients());
            }
            ArticulatorPosterior infer = ArticulatorPosterior.INSTANCE.infer(areaFromCoefficients, this.asset.getAreaMeanCm2());
            List<Float> list = trackFormants;
            ArrayList arrayList = new ArrayList(CollectionsKt.collectionSizeOrDefault(list, 10));
            it = list.iterator();
            while (it.hasNext()) {
                ((Number) it.next()).floatValue();
                arrayList.add(Float.valueOf(((acceptedF0Hz != null ? acceptedF0Hz.floatValue() : 0.0f) * 0.18f) + 75.0f + ((1.0f - coerceIn2) * 190.0f)));
            }
            ArrayList arrayList2 = arrayList;
            if (acceptedF0Hz == null) {
                f4 = 1.0f;
                f5 = Float.valueOf(Math.max(0.5f, acceptedF0Hz.floatValue() * 0.07f * (1.0f - floatValue)));
            } else {
                f4 = 1.0f;
                f5 = null;
            }
            float nanoTime2 = (System.nanoTime() - nanoTime) / 1000000.0f;
            long timestampNanos = frame.getTimestampNanos();
            String source = frame.getSource();
            AcousticEstimate acousticEstimate = new AcousticEstimate(acceptedF0Hz == null ? z : false, acceptedF0Hz, floatValue, f5, trackFormants, arrayList2, emptyList, f, f2, update3.getDecision(), update3.getRejected(), f3, update2.getSnrDb(), update2.getNoiseFloorDb(), update2.getState(), update2.getConfidence(), update2.getChanged(), update2.getBandsDb(), component2);
            float[] sectionPosition = this.asset.getSectionPosition();
            float[] copyOf = Arrays.copyOf(sectionPosition, sectionPosition.length);
            Intrinsics.checkNotNullExpressionValue(copyOf, str);
            return new VocalAcousticsState(sequence, timestampNanos, source, nanoTime2, droppedFrames, acousticEstimate, new TractPosterior(copyOf, areaFromCoefficients, update.getCoefficients(), update.getConfidence(), ((f4 - update.getConfidence()) * 0.53f) + 0.12f, null, update.getAbstained(), update.getReason(), this.asset.getModeLabels(), 32, null), infer, (this.asset.getFrameSize() * 1000.0f) / this.asset.getSampleRateHz(), (this.asset.getHopSize() * 1000.0f) / this.asset.getSampleRateHz());
        }
        z2 = false;
        TemporalAtlasFilter temporalAtlasFilter2 = this.temporalFilter;
        if (z2) {
        }
        if (acceptedF0Hz != null) {
        }
        f3 = harmonicity;
        z3 = false;
        update = temporalAtlasFilter2.update(second, coerceIn3, z3);
        if (z2) {
            update = TemporalAtlasEstimate.copy$default(update, null, 0.0f, false, component2.size() >= 3 ? "insufficient_formant_peaks" : "model_mismatch", 7, null);
        }
        sharedTractModel = this.sharedModel;
        if (sharedTractModel != null) {
        }
        float[] areaFromCoefficients2 = this.asset.areaFromCoefficients(update.getCoefficients());
        ArticulatorPosterior infer2 = ArticulatorPosterior.INSTANCE.infer(areaFromCoefficients2, this.asset.getAreaMeanCm2());
        List<Float> list2 = trackFormants;
        ArrayList arrayList3 = new ArrayList(CollectionsKt.collectionSizeOrDefault(list2, 10));
        it = list2.iterator();
        while (it.hasNext()) {
        }
        ArrayList arrayList22 = arrayList3;
        if (acceptedF0Hz == null) {
        }
        float nanoTime22 = (System.nanoTime() - nanoTime) / 1000000.0f;
        long timestampNanos2 = frame.getTimestampNanos();
        String source2 = frame.getSource();
        if (acceptedF0Hz == null) {
        }
        AcousticEstimate acousticEstimate2 = new AcousticEstimate(acceptedF0Hz == null ? z : false, acceptedF0Hz, floatValue, f5, trackFormants, arrayList22, emptyList, f, f2, update3.getDecision(), update3.getRejected(), f3, update2.getSnrDb(), update2.getNoiseFloorDb(), update2.getState(), update2.getConfidence(), update2.getChanged(), update2.getBandsDb(), component2);
        float[] sectionPosition2 = this.asset.getSectionPosition();
        float[] copyOf2 = Arrays.copyOf(sectionPosition2, sectionPosition2.length);
        Intrinsics.checkNotNullExpressionValue(copyOf2, str);
        return new VocalAcousticsState(sequence, timestampNanos2, source2, nanoTime22, droppedFrames, acousticEstimate2, new TractPosterior(copyOf2, areaFromCoefficients2, update.getCoefficients(), update.getConfidence(), ((f4 - update.getConfidence()) * 0.53f) + 0.12f, null, update.getAbstained(), update.getReason(), this.asset.getModeLabels(), 32, null), infer2, (this.asset.getFrameSize() * 1000.0f) / this.asset.getSampleRateHz(), (this.asset.getHopSize() * 1000.0f) / this.asset.getSampleRateHz());
    }

    private final Pair<Float, Float> estimateF0(float[] samples) {
        int i;
        int i2;
        FrameAnalyzer frameAnalyzer = this;
        int max = Math.max(1, frameAnalyzer.sampleRate / 700);
        int min = Math.min(frameAnalyzer.frameSize - 2, frameAnalyzer.sampleRate / 60);
        float[] fArr = new float[min + 1];
        if (max <= min) {
            int i3 = max;
            while (true) {
                int i4 = frameAnalyzer.frameSize - i3;
                double d = 0.0d;
                double d2 = 0.0d;
                int i5 = 0;
                double d3 = 0.0d;
                while (i5 < i4) {
                    double d4 = samples[i5];
                    double d5 = samples[i5 + i3];
                    d += d4 * d5;
                    d3 += d4 * d4;
                    d2 += d5 * d5;
                    i5++;
                    max = max;
                }
                i = max;
                fArr[i3] = (float) (d / Math.sqrt(Math.max(1.0E-12d, d3 * d2)));
                if (i3 == min) {
                    break;
                }
                i3++;
                frameAnalyzer = this;
                max = i;
            }
        } else {
            i = max;
        }
        int i6 = i + 1;
        if (i6 <= min) {
            int i7 = i6;
            i2 = i;
            while (true) {
                if (fArr[i7] > fArr[i2]) {
                    i2 = i7;
                }
                if (i7 == min) {
                    break;
                }
                i7++;
            }
        } else {
            i2 = i;
        }
        float max2 = Math.max(0.55f, fArr[i2] * 0.9f);
        while (true) {
            if (i6 >= min) {
                break;
            }
            float f = fArr[i6];
            if (f >= max2 && f >= fArr[i6 - 1] && f > fArr[i6 + 1]) {
                i2 = i6;
                break;
            }
            i6++;
        }
        float coerceIn = RangesKt.coerceIn(fArr[i2], 0.0f, 1.0f);
        if (coerceIn < 0.3f) {
            return new Pair<>(null, Float.valueOf(coerceIn));
        }
        float f2 = i2;
        if (i2 > i && i2 < min) {
            float f3 = fArr[i2 - 1];
            float f4 = fArr[i2];
            float f5 = fArr[i2 + 1];
            float f6 = (f3 - (f4 * 2.0f)) + f5;
            if (Math.abs(f6) > 1.0E-7f) {
                f2 += ((f3 - f5) * 0.5f) / f6;
            }
        }
        return new Pair<>(Float.valueOf(this.sampleRate / f2), Float.valueOf(coerceIn));
    }

    private final float[] spectrumPower(float[] samples) {
        int i = this.frameSize;
        double[] dArr = new double[i];
        double[] dArr2 = new double[i];
        int length = samples.length;
        for (int i2 = 0; i2 < length; i2++) {
            dArr[i2] = samples[i2] * this.hann[i2];
        }
        RealFft.INSTANCE.transform(dArr, dArr2);
        int i3 = (this.frameSize / 2) + 1;
        float[] fArr = new float[i3];
        for (int i4 = 0; i4 < i3; i4++) {
            double d = dArr[i4];
            double d2 = dArr2[i4];
            fArr[i4] = (float) Math.max(1.0E-12d, (d * d) + (d2 * d2));
        }
        return fArr;
    }

    private final float[] relativeDb(float[] power) {
        int length = power.length;
        float[] fArr = new float[length];
        int length2 = power.length;
        float f = -240.0f;
        for (int i = 0; i < length2; i++) {
            float log10 = (float) (Math.log10(Math.max(1.0E-12d, power[i])) * 10.0d);
            fArr[i] = log10;
            f = Math.max(f, log10);
        }
        for (int i2 = 0; i2 < length; i2++) {
            fArr[i2] = fArr[i2] - f;
        }
        return fArr;
    }

    private final float harmonicity(Float f0, float[] power) {
        if (f0 == null) {
            return 0.0f;
        }
        float f = this.sampleRate / this.frameSize;
        int min = Math.min(ArraysKt.getLastIndex(power), (int) (5500.0f / f));
        double d = 0.0d;
        double d2 = 0.0d;
        if (1 <= min) {
            int i = 1;
            while (true) {
                d2 += power[i];
                if (i == min) {
                    break;
                }
                i++;
            }
        }
        int i2 = 1;
        while (true) {
            float f2 = i2;
            if (f0.floatValue() * f2 < 5500.0f) {
                int coerceIn = RangesKt.coerceIn(MathKt.roundToInt((f2 * f0.floatValue()) / f), 1, min);
                for (int i3 = -1; i3 < 2; i3++) {
                    d += power[RangesKt.coerceIn(coerceIn + i3, 1, min)];
                }
                i2++;
            } else {
                return RangesKt.coerceIn((float) (d / Math.max(1.0E-12d, d2)), 0.0f, 1.0f);
            }
        }
    }

    private final Pair<List<Float>, List<Float>> estimateFormants(float[] spectrumDb, float f0) {
        float f = this.sampleRate / this.frameSize;
        int max = Math.max(1, (int) (Math.max(100.0f, 0.8f * f0) / f));
        float[] fArr = new float[spectrumDb.length];
        int length = spectrumDb.length;
        for (int i = 0; i < length; i++) {
            int i2 = -max;
            float f2 = 0.0f;
            float f3 = 0.0f;
            if (i2 <= max) {
                while (true) {
                    int i3 = i + i2;
                    if (i3 >= 0 && i3 < spectrumDb.length) {
                        double d = i2;
                        float exp = (float) Math.exp((((-0.5d) * d) * d) / Math.max(1.0f, (max * max) / 4.0f));
                        f2 += spectrumDb[i3] * exp;
                        f3 += exp;
                    }
                    if (i2 != max) {
                        i2++;
                    }
                }
            }
            fArr[i] = f2 / Math.max(1.0E-6f, f3);
        }
        int max2 = Math.max(1, (int) (180.0f / f));
        int min = Math.min(ArraysKt.getLastIndex(fArr) - 1, (int) (5000.0f / f));
        ArrayList arrayList = new ArrayList();
        if (max2 <= min) {
            int i4 = max2;
            while (true) {
                float f4 = fArr[i4];
                if (f4 > fArr[i4 - 1] && f4 >= fArr[i4 + 1]) {
                    int i5 = max * 2;
                    float min2 = fArr[i4] - Math.min(fArr[Math.max(max2, i4 - i5)], fArr[Math.min(min, i5 + i4)]);
                    if (min2 >= 0.25f) {
                        arrayList.add(new Pair(Integer.valueOf(i4), Float.valueOf(min2)));
                    }
                }
                if (i4 == min) {
                    break;
                }
                i4++;
            }
        }
        ArrayList arrayList2 = new ArrayList();
        Iterator it = CollectionsKt.sortedWith(arrayList, new Comparator() { // from class: org.vocaltract.pixel.FrameAnalyzer$estimateFormants$$inlined$sortedByDescending$1
            /* JADX DEBUG: Multi-variable search result rejected for r1v0, resolved type: T */
            /* JADX DEBUG: Multi-variable search result rejected for r2v0, resolved type: T */
            /* JADX WARN: Multi-variable type inference failed */
            @Override // java.util.Comparator
            public final int compare(T t, T t2) {
                return ComparisonsKt.compareValues((Float) ((Pair) t2).getSecond(), (Float) ((Pair) t).getSecond());
            }
        }).iterator();
        while (it.hasNext()) {
            int intValue = ((Number) ((Pair) it.next()).component1()).intValue();
            ArrayList arrayList3 = arrayList2;
            if (!(arrayList3 instanceof Collection) || !arrayList3.isEmpty()) {
                Iterator it2 = arrayList3.iterator();
                while (it2.hasNext()) {
                    if (Math.abs(((Number) it2.next()).intValue() - intValue) * f < Math.max(220.0f, 0.75f * f0)) {
                        break;
                    }
                }
            }
            arrayList2.add(Integer.valueOf(intValue));
            if (arrayList2.size() >= 10) {
                break;
            }
        }
        List sorted = CollectionsKt.sorted(arrayList2);
        ArrayList arrayList4 = new ArrayList(CollectionsKt.collectionSizeOrDefault(sorted, 10));
        Iterator it3 = sorted.iterator();
        while (it3.hasNext()) {
            arrayList4.add(Float.valueOf(((Number) it3.next()).intValue() * f));
        }
        ArrayList arrayList5 = arrayList4;
        List mutableList = CollectionsKt.toMutableList((Collection) CollectionsKt.take(arrayList5, this.asset.getReferenceFormantsHz().length));
        while (mutableList.size() < this.asset.getReferenceFormantsHz().length) {
            mutableList.add(Float.valueOf(this.asset.getReferenceFormantsHz()[mutableList.size()]));
        }
        return new Pair<>(mutableList, arrayList5);
    }

    private final List<Float> trackFormants(List<Float> observed, Float f0, float f0Confidence, float snrDb) {
        float[] floatArray = CollectionsKt.toFloatArray(observed);
        float[] fArr = this.trackedFormants;
        if (fArr == null || f0 == null) {
            this.trackedFormants = floatArray;
            return ArraysKt.toList(floatArray);
        }
        float coerceIn = RangesKt.coerceIn((f0Confidence * 0.22f) + 0.1f + (snrDb * 0.012f), 0.1f, 0.42f);
        int length = floatArray.length;
        for (int i = 0; i < length; i++) {
            float f = fArr[i];
            float f2 = f + ((floatArray[i] - f) * coerceIn);
            floatArray[i] = f2;
            if (i > 0) {
                floatArray[i] = Math.max(f2, floatArray[i - 1] + 180.0f);
            }
        }
        this.trackedFormants = floatArray;
        return ArraysKt.toList(floatArray);
    }

    private final List<HarmonicEstimate> buildHarmonics(float f0, float[] spectrumDb) {
        float f = this.sampleRate / this.frameSize;
        List createListBuilder = CollectionsKt.createListBuilder();
        int i = this.maxHarmonics;
        int i2 = 1;
        if (1 <= i) {
            while (true) {
                float f2 = i2 * f0;
                if (f2 >= this.sampleRate / 2.0f) {
                    break;
                }
                float coerceIn = RangesKt.coerceIn(f2 / f, 0.0f, ArraysKt.getLastIndex(spectrumDb));
                int i3 = (int) coerceIn;
                float f3 = coerceIn - i3;
                createListBuilder.add(new HarmonicEstimate(i2, f2, (spectrumDb[i3] * (1.0f - f3)) + (spectrumDb[Math.min(i3 + 1, ArraysKt.getLastIndex(spectrumDb))] * f3)));
                if (i2 == i) {
                    break;
                }
                i2++;
            }
        }
        return CollectionsKt.build(createListBuilder);
    }
}

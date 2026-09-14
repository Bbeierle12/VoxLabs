package org.vocaltract.pixel;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.Iterator;
import java.util.List;
import kotlin.Metadata;
import kotlin.collections.ArraysKt;
import kotlin.collections.CollectionsKt;
import kotlin.collections.IntIterator;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.IntRange;
import kotlin.ranges.RangesKt;
import org.vocaltract.pixel.AdditiveSynthState;

/* compiled from: SharedTractModel.kt */
@Metadata(d1 = {"\u0000b\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0007\n\u0002\u0010\u000e\n\u0002\b\u0003\n\u0002\u0010\u0013\n\u0002\b\u0003\n\u0002\u0010\u0006\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0010\u0014\n\u0002\b\u0004\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0010\b\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0003\u0018\u00002\u00020\u0001B\u0017\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005¢\u0006\u0004\b\u0006\u0010\u0007J\u000e\u0010\u001c\u001a\u00020\u001d2\u0006\u0010\u001e\u001a\u00020\u001dJ\u000e\u0010\u001f\u001a\u00020\u001d2\u0006\u0010\u001e\u001a\u00020\u001dJ\u000e\u0010 \u001a\u00020\u001d2\u0006\u0010\u001e\u001a\u00020\u001dJ\u000e\u0010!\u001a\u00020\"2\u0006\u0010\u001e\u001a\u00020\u001dJ\"\u0010#\u001a\u00020$2\u0006\u0010%\u001a\u00020\u001d2\b\b\u0002\u0010&\u001a\u00020\u001d2\b\b\u0002\u0010'\u001a\u00020(J&\u0010)\u001a\u00020*2\u0006\u0010\u001e\u001a\u00020\u001d2\u0006\u0010+\u001a\u00020,2\u0006\u0010-\u001a\u00020\u001d2\u0006\u0010.\u001a\u00020,J\u000e\u0010/\u001a\u00020(2\u0006\u0010\u001e\u001a\u00020\u001dR\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\b\u0010\tR\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\n\u0010\u000bR\u0014\u0010\f\u001a\u00020\rX\u0086D¢\u0006\b\n\u0000\u001a\u0004\b\u000e\u0010\u000fR\u0011\u0010\u0010\u001a\u00020\u0011¢\u0006\b\n\u0000\u001a\u0004\b\u0012\u0010\u0013R\u0011\u0010\u0014\u001a\u00020\u0015¢\u0006\b\n\u0000\u001a\u0004\b\u0016\u0010\u0017R\u0011\u0010\u0018\u001a\u00020\u0019¢\u0006\b\n\u0000\u001a\u0004\b\u001a\u0010\u001b"}, d2 = {"Lorg/vocaltract/pixel/SharedTractModel;", "", "atlas", "Lorg/vocaltract/pixel/ReducedModelAsset;", "lumen", "Lorg/vocaltract/pixel/TractLumenAsset;", "<init>", "(Lorg/vocaltract/pixel/ReducedModelAsset;Lorg/vocaltract/pixel/TractLumenAsset;)V", "getAtlas", "()Lorg/vocaltract/pixel/ReducedModelAsset;", "getLumen", "()Lorg/vocaltract/pixel/TractLumenAsset;", "id", "", "getId", "()Ljava/lang/String;", "lengthsMm", "", "getLengthsMm", "()[D", "lengthMm", "", "getLengthMm", "()D", "grid", "Lorg/vocaltract/pixel/TubeGrid;", "getGrid", "()Lorg/vocaltract/pixel/TubeGrid;", "checked", "", "input", "area", "vertices", "response", "Lorg/vocaltract/pixel/TractResponse;", "fit", "Lorg/vocaltract/pixel/TractFit;", "targets", "seed", "passes", "", "sound", "Lorg/vocaltract/pixel/AdditiveSynthState;", "f0", "", "source", "master", "clampedSections"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class SharedTractModel {
    private final ReducedModelAsset atlas;
    private final TubeGrid grid;
    private final String id;
    private final double lengthMm;
    private final double[] lengthsMm;
    private final TractLumenAsset lumen;

    public SharedTractModel(ReducedModelAsset atlas, TractLumenAsset lumen) {
        Intrinsics.checkNotNullParameter(atlas, "atlas");
        Intrinsics.checkNotNullParameter(lumen, "lumen");
        this.atlas = atlas;
        this.lumen = lumen;
        this.id = "shared-mri-airway-v1";
        int sectionCount = lumen.getSectionCount() - 1;
        double[] dArr = new double[sectionCount];
        int i = 0;
        while (true) {
            double d = 0.0d;
            if (i >= sectionCount) {
                break;
            }
            Iterator<Integer> it = new IntRange(0, 2).iterator();
            while (it.hasNext()) {
                int nextInt = ((IntIterator) it).nextInt();
                d += Math.pow(this.lumen.getCenterlineMm()[((i + 1) * 3) + nextInt] - this.lumen.getCenterlineMm()[(i * 3) + nextInt], 2);
            }
            dArr[i] = Math.sqrt(d);
            i++;
        }
        this.lengthsMm = dArr;
        this.lengthMm = ArraysKt.sum(dArr);
        double[] dArr2 = new double[497];
        for (int i2 = 0; i2 < 497; i2++) {
            dArr2[i2] = (i2 * 10.0d) + 40.0d;
        }
        this.grid = new TubeGrid(dArr, dArr2);
        if (this.atlas.getSchemaVersion() != 2 || this.lumen.getProvenanceKind() != 2) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (this.atlas.getAreaMeanCm2().length != this.lumen.getSectionCount() || this.atlas.getAreaModes().length != 4) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        Iterable indices = ArraysKt.getIndices(this.atlas.getAreaMeanCm2());
        if (!(indices instanceof Collection) || !((Collection) indices).isEmpty()) {
            Iterator it2 = indices.iterator();
            while (it2.hasNext()) {
                int nextInt2 = ((IntIterator) it2).nextInt();
                if (Math.abs(this.atlas.getAreaMeanCm2()[nextInt2] - this.lumen.getReferenceAreaCm2()[nextInt2]) >= 1.0E-5f) {
                    throw new IllegalArgumentException("Failed requirement.".toString());
                }
            }
        }
        Iterable indices2 = ArraysKt.getIndices(this.atlas.getSectionPosition());
        if (!(indices2 instanceof Collection) || !((Collection) indices2).isEmpty()) {
            Iterator it3 = indices2.iterator();
            while (it3.hasNext()) {
                int nextInt3 = ((IntIterator) it3).nextInt();
                if (Math.abs(this.atlas.getSectionPosition()[nextInt3] - this.lumen.getSectionPosition()[nextInt3]) >= 1.0E-5f) {
                    throw new IllegalArgumentException("Failed requirement.".toString());
                }
            }
        }
        for (double d2 : this.lengthsMm) {
            if (Double.isInfinite(d2) || Double.isNaN(d2) || d2 <= 0.0d) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
        }
        if (!this.lumen.getSourceSha256().contains(this.atlas.getAtlasSourceModelSha256())) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (this.atlas.getScientificReleaseReady() || this.atlas.getIndependentExpertAcceptances() != 0) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
    }

    public final ReducedModelAsset getAtlas() {
        return this.atlas;
    }

    public final TractLumenAsset getLumen() {
        return this.lumen;
    }

    public final String getId() {
        return this.id;
    }

    public final double[] getLengthsMm() {
        return this.lengthsMm;
    }

    public final double getLengthMm() {
        return this.lengthMm;
    }

    public final TubeGrid getGrid() {
        return this.grid;
    }

    public final float[] checked(float[] input) {
        int i;
        Intrinsics.checkNotNullParameter(input, "input");
        if (input.length == 4) {
            for (float f : input) {
                i = (Float.isInfinite(f) || Float.isNaN(f)) ? 0 : i + 1;
            }
            Iterable indices = ArraysKt.getIndices(input);
            if (!(indices instanceof Collection) || !((Collection) indices).isEmpty()) {
                Iterator it = indices.iterator();
                while (it.hasNext()) {
                    int nextInt = ((IntIterator) it).nextInt();
                    float f2 = this.atlas.getCoefficientLimitsSd()[nextInt][0];
                    float f3 = this.atlas.getCoefficientLimitsSd()[nextInt][1];
                    float f4 = input[nextInt];
                    if (f2 > f4 || f4 > f3) {
                        throw new IllegalArgumentException("MRI coordinate outside model limits".toString());
                    }
                }
            }
            float[] copyOf = Arrays.copyOf(input, input.length);
            Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
            return copyOf;
        }
        throw new IllegalArgumentException("Four finite MRI coordinates required".toString());
    }

    public final float[] area(float[] input) {
        Intrinsics.checkNotNullParameter(input, "input");
        return this.atlas.areaFromCoefficients(checked(input));
    }

    public final float[] vertices(float[] input) {
        Intrinsics.checkNotNullParameter(input, "input");
        return this.lumen.morph(area(input));
    }

    public final TractResponse response(float[] input) {
        Intrinsics.checkNotNullParameter(input, "input");
        return this.grid.response(area(input));
    }

    public static /* synthetic */ TractFit fit$default(SharedTractModel sharedTractModel, float[] fArr, float[] fArr2, int i, int i2, Object obj) {
        if ((i2 & 2) != 0) {
            fArr2 = new float[4];
        }
        if ((i2 & 4) != 0) {
            i = 5;
        }
        return sharedTractModel.fit(fArr, fArr2, i);
    }

    /* JADX DEBUG: Multi-variable search result rejected for r22v0, resolved type: boolean */
    /* JADX DEBUG: Multi-variable search result rejected for r22v1, resolved type: boolean */
    /* JADX DEBUG: Multi-variable search result rejected for r22v2, resolved type: boolean */
    /* JADX WARN: Multi-variable type inference failed */
    public final TractFit fit(float[] targets, float[] seed, int passes) {
        int i;
        float f;
        boolean z;
        Intrinsics.checkNotNullParameter(targets, "targets");
        Intrinsics.checkNotNullParameter(seed, "seed");
        int length = targets.length;
        if (3 <= length && length < 5) {
            int i2 = 0;
            for (float f2 : targets) {
                i = (!Float.isInfinite(f2) && !Float.isNaN(f2) && 150.0f <= f2 && f2 <= 4900.0f) ? i + 1 : 0;
            }
            int i3 = 1;
            Iterable until = RangesKt.until(1, targets.length);
            if (!(until instanceof Collection) || !((Collection) until).isEmpty()) {
                Iterator it = until.iterator();
                while (it.hasNext()) {
                    int nextInt = ((IntIterator) it).nextInt();
                    if (targets[nextInt] <= targets[nextInt - 1]) {
                        throw new IllegalArgumentException("Formant targets must increase".toString());
                    }
                }
            }
            if (1 > passes || passes >= 7) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            float[] checked = checked(seed);
            float[] copyOf = Arrays.copyOf(checked, checked.length);
            Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
            double fit$error = fit$error(this, targets, copyOf);
            int i4 = 4;
            if (passes >= 4) {
                for (float[] fArr : CollectionsKt.listOf(new float[4], this.atlas.inferArea(ArraysKt.toList(targets)).getSecond())) {
                    double fit$error2 = fit$error(this, targets, fArr);
                    if (fit$error2 < fit$error) {
                        copyOf = Arrays.copyOf(fArr, fArr.length);
                        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
                        fit$error = fit$error2;
                    }
                }
            }
            double[] dArr = passes >= 4 ? new double[]{0.8d, 0.4d, 0.2d, 0.1d, 0.05d, 0.025d} : new double[]{0.15d, 0.075d, 0.04d};
            int i5 = 0;
            while (i5 < passes) {
                int i6 = i2;
                while (i6 < i4) {
                    Iterator it2 = CollectionsKt.listOf((Object[]) new Integer[]{-1, Integer.valueOf(i3)}).iterator();
                    while (it2.hasNext()) {
                        int intValue = ((Number) it2.next()).intValue();
                        float[] copyOf2 = Arrays.copyOf(copyOf, copyOf.length);
                        Intrinsics.checkNotNullExpressionValue(copyOf2, "copyOf(...)");
                        double d = fit$error;
                        copyOf2[i6] = RangesKt.coerceIn((float) (copyOf2[i6] + (intValue * dArr[i5])), this.atlas.getCoefficientLimitsSd()[i6][0], this.atlas.getCoefficientLimitsSd()[i6][1]);
                        fit$error = fit$error(this, targets, copyOf2);
                        i2 = 0;
                        if (fit$error < d) {
                            copyOf = copyOf2;
                            i3 = 1;
                        } else {
                            i3 = 1;
                            fit$error = d;
                        }
                    }
                    i6++;
                    i3 = i3;
                    i4 = 4;
                }
                i5++;
                i3 = i3;
                i4 = 4;
            }
            int i7 = i3;
            int i8 = i2;
            List<Double> take = ArraysKt.take(response(copyOf).getPeaksHz(), targets.length);
            ArrayList arrayList = new ArrayList(CollectionsKt.collectionSizeOrDefault(take, 10));
            Iterator<T> it3 = take.iterator();
            while (it3.hasNext()) {
                arrayList.add(Float.valueOf((float) ((Number) it3.next()).doubleValue()));
            }
            float[] floatArray = CollectionsKt.toFloatArray(arrayList);
            if (floatArray.length == targets.length) {
                Iterator<Integer> it4 = ArraysKt.getIndices(targets).iterator();
                double d2 = 0.0d;
                while (it4.hasNext()) {
                    int nextInt2 = ((IntIterator) it4).nextInt();
                    d2 += Math.pow(floatArray[nextInt2] - targets[nextInt2], 2);
                }
                f = (float) Math.sqrt(d2 / targets.length);
            } else {
                f = Float.POSITIVE_INFINITY;
            }
            float f3 = f;
            float[] copyOf3 = Arrays.copyOf(copyOf, copyOf.length);
            Intrinsics.checkNotNullExpressionValue(copyOf3, "copyOf(...)");
            float[] copyOf4 = Arrays.copyOf(targets, targets.length);
            Intrinsics.checkNotNullExpressionValue(copyOf4, "copyOf(...)");
            Iterable indices = ArraysKt.getIndices(targets);
            if (!(indices instanceof Collection) || !((Collection) indices).isEmpty()) {
                Iterator it5 = indices.iterator();
                while (it5.hasNext()) {
                    int nextInt3 = ((IntIterator) it5).nextInt();
                    if (nextInt3 >= floatArray.length || Math.abs(floatArray[nextInt3] - targets[nextInt3]) > 100.0f) {
                        z = i8;
                        break;
                    }
                }
            }
            z = i7;
            return new TractFit(copyOf3, copyOf4, floatArray, f3, z);
        }
        throw new IllegalArgumentException("Failed requirement.".toString());
    }

    private static final double fit$error(SharedTractModel sharedTractModel, float[] fArr, float[] fArr2) {
        double[] peaksHz = sharedTractModel.response(fArr2).getPeaksHz();
        if (peaksHz.length < fArr.length) {
            return 1.0E9d;
        }
        Iterator<Integer> it = ArraysKt.getIndices(fArr).iterator();
        double d = 0.0d;
        double d2 = 0.0d;
        while (it.hasNext()) {
            int nextInt = ((IntIterator) it).nextInt();
            d2 += Math.pow((peaksHz[nextInt] - fArr[nextInt]) / ((nextInt * 80.0d) + 100.0d), 2);
        }
        for (double d3 : fArr2) {
            d += 0.002d * d3 * d3;
        }
        return d2 + d;
    }

    public final AdditiveSynthState sound(float[] input, float f0, float[] source, float master) {
        int i;
        Intrinsics.checkNotNullParameter(input, "input");
        Intrinsics.checkNotNullParameter(source, "source");
        if (Float.isInfinite(f0) || Float.isNaN(f0) || 55.0f > f0 || f0 > 800.0f) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (source.length == 16) {
            for (float f : source) {
                i = (!Float.isInfinite(f) && !Float.isNaN(f) && 0.0f <= f && f <= 1.0f) ? i + 1 : 0;
            }
            if (Float.isInfinite(master) || Float.isNaN(master) || 0.0f > master || master > 0.92f) {
                throw new IllegalArgumentException("Failed requirement.".toString());
            }
            double[] dArr = this.lengthsMm;
            double[] dArr2 = new double[16];
            int i2 = 0;
            while (i2 < 16) {
                int i3 = i2 + 1;
                dArr2[i2] = f0 * i3;
                i2 = i3;
            }
            double[] db = new TubeGrid(dArr, dArr2).response(area(input)).getDb();
            float[] fArr = new float[16];
            for (int i4 = 0; i4 < 16; i4++) {
                fArr[i4] = source[i4] * ((float) Math.pow(10.0d, db[i4] / 20.0d));
            }
            AdditiveSynthState.Companion companion = AdditiveSynthState.INSTANCE;
            ArrayList arrayList = new ArrayList(16);
            for (int i5 = 0; i5 < 16; i5++) {
                arrayList.add(Float.valueOf(RangesKt.coerceIn(fArr[i5], 0.0f, 1.0f)));
            }
            return companion.create(f0, master, arrayList, false);
        }
        throw new IllegalArgumentException("Failed requirement.".toString());
    }

    public final int clampedSections(float[] input) {
        Intrinsics.checkNotNullParameter(input, "input");
        float[] area = area(input);
        Iterable indices = ArraysKt.getIndices(area);
        int i = 0;
        if (!(indices instanceof Collection) || !((Collection) indices).isEmpty()) {
            Iterator it = indices.iterator();
            while (it.hasNext()) {
                int nextInt = ((IntIterator) it).nextInt();
                if (area[nextInt] <= this.atlas.getAreaMeanCm2()[nextInt] * 0.20001f || area[nextInt] >= this.atlas.getAreaMeanCm2()[nextInt] * 4.9999f) {
                    i++;
                    if (i < 0) {
                        CollectionsKt.throwCountOverflow();
                    }
                }
            }
        }
        return i;
    }
}

package org.vocaltract.pixel;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.Path;
import android.graphics.RectF;
import android.os.SystemClock;
import android.util.AttributeSet;
import android.view.View;
import java.util.Arrays;
import kotlin.Metadata;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;
import org.vocaltract.pixel.AdditiveSynthState;

/* compiled from: SummedWaveformView.kt */
@Metadata(d1 = {"\u0000l\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\b\u0006\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0014\n\u0000\n\u0002\u0010\t\n\u0000\n\u0002\u0010\u000b\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0010\u0002\n\u0002\b\u0006\n\u0002\u0010\b\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\b\u0005\n\u0002\u0010\u0007\n\u0000\n\u0002\u0010\u0006\n\u0002\b\u0003\u0018\u0000 32\u00020\u0001:\u00013B\u001d\b\u0007\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\n\b\u0002\u0010\u0004\u001a\u0004\u0018\u00010\u0005¢\u0006\u0004\b\u0006\u0010\u0007J\u000e\u0010\u001c\u001a\u00020\u001d2\u0006\u0010\u001e\u001a\u00020\u0010J\u000e\u0010\u001f\u001a\u00020\u001d2\u0006\u0010 \u001a\u00020\u0012J\u000e\u0010!\u001a\u00020\u001d2\u0006\u0010\u001e\u001a\u00020\u0016J\u0018\u0010\"\u001a\u00020\u001d2\u0006\u0010#\u001a\u00020$2\u0006\u0010%\u001a\u00020$H\u0014J\u0010\u0010&\u001a\u00020\u001d2\u0006\u0010'\u001a\u00020(H\u0014J\u0010\u0010)\u001a\u00020\u001d2\u0006\u0010'\u001a\u00020(H\u0002J0\u0010*\u001a\u00020\u001d2\u0006\u0010'\u001a\u00020(2\u0006\u0010 \u001a\u00020\u00122\u0006\u0010+\u001a\u00020\u001a2\u0006\u0010,\u001a\u00020\t2\u0006\u0010-\u001a\u00020.H\u0002J\u0018\u0010/\u001a\u0002002\u0006\u00101\u001a\u00020\u00102\u0006\u00102\u001a\u00020\u0014H\u0002R\u000e\u0010\b\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\n\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u000b\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\f\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\r\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u000e\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u000f\u001a\u00020\u0010X\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010\u0011\u001a\u0004\u0018\u00010\u0012X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0013\u001a\u00020\u0014X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0015\u001a\u00020\u0016X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0017\u001a\u00020\u0018X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0019\u001a\u00020\u001aX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u001b\u001a\u00020\u001aX\u0082\u0004¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/SummedWaveformView;", "Landroid/view/View;", "context", "Landroid/content/Context;", "attrs", "Landroid/util/AttributeSet;", "<init>", "(Landroid/content/Context;Landroid/util/AttributeSet;)V", "backgroundPaint", "Landroid/graphics/Paint;", "gridPaint", "summedPaint", "standingPaint", "labelPaint", "standingLabelPaint", "synthState", "Lorg/vocaltract/pixel/AdditiveSynthState;", "latestSamples", "", "latestSampleTimeMs", "", "animationEnabled", "", "bounds", "Landroid/graphics/RectF;", "summedPath", "Landroid/graphics/Path;", "standingPath", "updateSynthState", "", "value", "updateSamples", "samples", "setAnimating", "onMeasure", "widthMeasureSpec", "", "heightMeasureSpec", "onDraw", "canvas", "Landroid/graphics/Canvas;", "drawGrid", "drawSignal", "path", "paint", "amplitudeScale", "", "animationPhase", "", "state", "nowMs", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class SummedWaveformView extends View {
    private static final Companion Companion = new Companion(null);

    @Deprecated
    public static final int DISPLAY_POINT_COUNT = 320;

    @Deprecated
    public static final int DISPLAY_SAMPLE_RATE_HZ = 48000;

    @Deprecated
    public static final long LIVE_SAMPLE_TIMEOUT_MS = 250;
    private volatile boolean animationEnabled;
    private final Paint backgroundPaint;
    private final RectF bounds;
    private final Paint gridPaint;
    private final Paint labelPaint;
    private volatile long latestSampleTimeMs;
    private volatile float[] latestSamples;
    private final Paint standingLabelPaint;
    private final Paint standingPaint;
    private final Path standingPath;
    private final Paint summedPaint;
    private final Path summedPath;
    private volatile AdditiveSynthState synthState;

    /* JADX DEBUG: Multi-variable search result rejected for r0v2, resolved type: java.lang.Object[] */
    /* JADX WARN: 'this' call moved to the top of the method (can break code semantics) */
    /* JADX WARN: Multi-variable type inference failed */
    public SummedWaveformView(Context context) {
        this(context, null, 2, 0 == true ? 1 : 0);
        Intrinsics.checkNotNullParameter(context, "context");
    }

    public /* synthetic */ SummedWaveformView(Context context, AttributeSet attributeSet, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(context, (i & 2) != 0 ? null : attributeSet);
    }

    /* JADX WARN: 'super' call moved to the top of the method (can break code semantics) */
    public SummedWaveformView(Context context, AttributeSet attributeSet) {
        super(context, attributeSet);
        Intrinsics.checkNotNullParameter(context, "context");
        Paint paint = new Paint(1);
        paint.setColor(Color.rgb(248, 252, 250));
        paint.setStyle(Paint.Style.FILL);
        this.backgroundPaint = paint;
        Paint paint2 = new Paint(1);
        paint2.setColor(Color.rgb(216, 229, 226));
        paint2.setStrokeWidth(getResources().getDisplayMetrics().density);
        paint2.setStyle(Paint.Style.STROKE);
        this.gridPaint = paint2;
        Paint paint3 = new Paint(1);
        paint3.setColor(Color.rgb(0, 105, 96));
        paint3.setStrokeWidth(getResources().getDisplayMetrics().density * 2.2f);
        paint3.setStrokeJoin(Paint.Join.ROUND);
        paint3.setStyle(Paint.Style.STROKE);
        this.summedPaint = paint3;
        Paint paint4 = new Paint(1);
        paint4.setColor(Color.argb(120, 22, 123, 185));
        paint4.setStrokeWidth(getResources().getDisplayMetrics().density * 1.4f);
        paint4.setStrokeJoin(Paint.Join.ROUND);
        paint4.setStyle(Paint.Style.STROKE);
        this.standingPaint = paint4;
        Paint paint5 = new Paint(1);
        paint5.setColor(Color.rgb(58, 74, 71));
        paint5.setTextSize(getResources().getDisplayMetrics().scaledDensity * 12.0f);
        this.labelPaint = paint5;
        Paint paint6 = new Paint(paint5);
        paint6.setColor(paint4.getColor());
        this.standingLabelPaint = paint6;
        this.synthState = AdditiveSynthState.Companion.create$default(AdditiveSynthState.INSTANCE, 0.0f, 0.0f, null, false, 15, null);
        this.latestSampleTimeMs = Long.MIN_VALUE;
        this.animationEnabled = true;
        this.bounds = new RectF();
        this.summedPath = new Path();
        this.standingPath = new Path();
        setContentDescription("Real-time sum of sixteen harmonic partials and standing-wave overlay");
        setBackgroundColor(0);
    }

    public final void updateSynthState(AdditiveSynthState value) {
        Intrinsics.checkNotNullParameter(value, "value");
        this.synthState = value;
        postInvalidateOnAnimation();
    }

    public final void updateSamples(float[] samples) {
        Intrinsics.checkNotNullParameter(samples, "samples");
        float[] copyOf = Arrays.copyOf(samples, samples.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        this.latestSamples = copyOf;
        this.latestSampleTimeMs = SystemClock.uptimeMillis();
        postInvalidateOnAnimation();
    }

    public final void setAnimating(boolean value) {
        this.animationEnabled = value;
        postInvalidateOnAnimation();
    }

    @Override // android.view.View
    protected void onMeasure(int widthMeasureSpec, int heightMeasureSpec) {
        setMeasuredDimension(View.resolveSize((int) (getResources().getDisplayMetrics().density * 320.0f), widthMeasureSpec), View.resolveSize((int) (getResources().getDisplayMetrics().density * 180.0f), heightMeasureSpec));
    }

    @Override // android.view.View
    protected void onDraw(Canvas canvas) {
        Intrinsics.checkNotNullParameter(canvas, "canvas");
        super.onDraw(canvas);
        canvas.drawRect(0.0f, 0.0f, getWidth(), getHeight(), this.backgroundPaint);
        this.bounds.set(getPaddingLeft() + 16.0f, getPaddingTop() + 28.0f, (getWidth() - getPaddingRight()) - 16.0f, (getHeight() - getPaddingBottom()) - 22.0f);
        if (this.bounds.width() <= 1.0f || this.bounds.height() <= 1.0f) {
            return;
        }
        drawGrid(canvas);
        long uptimeMillis = SystemClock.uptimeMillis();
        AdditiveSynthState additiveSynthState = this.synthState;
        float[] fArr = this.latestSamples;
        if (fArr == null || uptimeMillis - this.latestSampleTimeMs > 250) {
            fArr = AdditiveSynthesisMath.INSTANCE.summedCycle(additiveSynthState, DISPLAY_SAMPLE_RATE_HZ, DISPLAY_POINT_COUNT, animationPhase(additiveSynthState, uptimeMillis));
        }
        float[] fArr2 = fArr;
        drawSignal(canvas, AdditiveSynthesisMath.INSTANCE.standingField(additiveSynthState, DISPLAY_SAMPLE_RATE_HZ, DISPLAY_POINT_COUNT, animationPhase(additiveSynthState, uptimeMillis)), this.standingPath, this.standingPaint, 0.72f);
        drawSignal(canvas, fArr2 == null ? new float[0] : fArr2, this.summedPath, this.summedPaint, 0.92f);
        canvas.drawText("16-partial sum • " + AdditiveSynthesisMath.activePartialCount$default(AdditiveSynthesisMath.INSTANCE, additiveSynthState.getFundamentalHz(), DISPLAY_SAMPLE_RATE_HZ, 0, 4, null) + " below Nyquist • F0 " + ((int) additiveSynthState.getFundamentalHz()) + " Hz", this.bounds.left, (getResources().getDisplayMetrics().scaledDensity * 19.0f) / getResources().getDisplayMetrics().density, this.labelPaint);
        canvas.drawText("standing field", this.bounds.left, ((float) getHeight()) - 6.0f, this.standingLabelPaint);
        canvas.drawText("summed PCM", this.bounds.right - this.labelPaint.measureText("summed PCM"), ((float) getHeight()) - 6.0f, this.labelPaint);
        if (this.animationEnabled && isAttachedToWindow()) {
            postInvalidateOnAnimation();
        }
    }

    private final void drawGrid(Canvas canvas) {
        for (int i = 0; i < 5; i++) {
            float width = this.bounds.left + ((this.bounds.width() * i) / 4.0f);
            canvas.drawLine(width, this.bounds.top, width, this.bounds.bottom, this.gridPaint);
        }
        canvas.drawLine(this.bounds.left, this.bounds.centerY(), this.bounds.right, this.bounds.centerY(), this.gridPaint);
        canvas.drawLine(this.bounds.left, this.bounds.top, this.bounds.right, this.bounds.top, this.gridPaint);
        canvas.drawLine(this.bounds.left, this.bounds.bottom, this.bounds.right, this.bounds.bottom, this.gridPaint);
    }

    private final void drawSignal(Canvas canvas, float[] samples, Path path, Paint paint, float amplitudeScale) {
        float width;
        if (samples.length == 0) {
            return;
        }
        float f = 0.05f;
        for (float f2 : samples) {
            f = Math.max(f, Math.abs(f2));
        }
        path.rewind();
        int length = samples.length;
        for (int i = 0; i < length; i++) {
            if (samples.length == 1) {
                width = this.bounds.left;
            } else {
                width = this.bounds.left + ((this.bounds.width() * i) / (samples.length - 1));
            }
            float centerY = this.bounds.centerY() - (((RangesKt.coerceIn(samples[i] / f, -1.0f, 1.0f) * this.bounds.height()) * 0.5f) * amplitudeScale);
            if (i == 0) {
                path.moveTo(width, centerY);
            } else {
                path.lineTo(width, centerY);
            }
        }
        canvas.drawPath(path, paint);
    }

    private final double animationPhase(AdditiveSynthState state, long nowMs) {
        if (!this.animationEnabled) {
            return 0.0d;
        }
        return ((RangesKt.coerceIn(state.getFundamentalHz(), 20.0f, 1000.0f) / 1000.0f) + 0.25d) * 6.283185307179586d * (nowMs / 1000.0d);
    }

    /* compiled from: SummedWaveformView.kt */
    @Metadata(d1 = {"\u0000\u0018\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\b\n\u0002\b\u0002\n\u0002\u0010\t\b\u0082\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003R\u000e\u0010\u0004\u001a\u00020\u0005X\u0086T¢\u0006\u0002\n\u0000R\u000e\u0010\u0006\u001a\u00020\u0005X\u0086T¢\u0006\u0002\n\u0000R\u000e\u0010\u0007\u001a\u00020\bX\u0086T¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/SummedWaveformView$Companion;", "", "<init>", "()V", "DISPLAY_SAMPLE_RATE_HZ", "", "DISPLAY_POINT_COUNT", "LIVE_SAMPLE_TIMEOUT_MS", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
    private static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }
    }
}

package org.vocaltract.pixel;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.Path;
import android.graphics.RectF;
import android.util.AttributeSet;
import android.view.View;
import java.util.Arrays;
import kotlin.Metadata;
import kotlin.collections.ArraysKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;
import kotlin.uuid.Uuid;

/* compiled from: TractSagittalView.kt */
@Metadata(d1 = {"\u0000X\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\b\u0005\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0002\n\u0002\b\u0003\n\u0002\u0010\b\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0014\n\u0002\b\u0002\n\u0002\u0010\u000b\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\u0018\u00002\u00020\u0001B\u001d\b\u0007\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\n\b\u0002\u0010\u0004\u001a\u0004\u0018\u00010\u0005¢\u0006\u0004\b\u0006\u0010\u0007J\u000e\u0010\u0010\u001a\u00020\u00112\u0006\u0010\u0012\u001a\u00020\u000fJ\u0018\u0010\u0013\u001a\u00020\u00112\u0006\u0010\u0014\u001a\u00020\u00152\u0006\u0010\u0016\u001a\u00020\u0015H\u0014J\u0010\u0010\u0017\u001a\u00020\u00112\u0006\u0010\u0018\u001a\u00020\u0019H\u0014J\u0018\u0010\u001a\u001a\u00020\u001b2\u0006\u0010\u001c\u001a\u00020\u000f2\u0006\u0010\u001d\u001a\u00020\u001eH\u0002J \u0010\u001f\u001a\u00020 2\u0006\u0010!\u001a\u00020\"2\u0006\u0010#\u001a\u00020\u001b2\u0006\u0010$\u001a\u00020\u001bH\u0002R\u000e\u0010\b\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\n\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u000b\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\f\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\r\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u0010\u0010\u000e\u001a\u0004\u0018\u00010\u000fX\u0082\u000e¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/TractSagittalView;", "Landroid/view/View;", "context", "Landroid/content/Context;", "attrs", "Landroid/util/AttributeSet;", "<init>", "(Landroid/content/Context;Landroid/util/AttributeSet;)V", "lumenPaint", "Landroid/graphics/Paint;", "uncertaintyPaint", "outlinePaint", "gridPaint", "labelPaint", "posterior", "Lorg/vocaltract/pixel/TractPosterior;", "updatePosterior", "", "value", "onMeasure", "widthMeasureSpec", "", "heightMeasureSpec", "onDraw", "canvas", "Landroid/graphics/Canvas;", "radii", "", "state", "includeUncertainty", "", "lumenPath", "Landroid/graphics/Path;", "bounds", "Landroid/graphics/RectF;", "sectionPosition", "normalizedRadius"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class TractSagittalView extends View {
    private final Paint gridPaint;
    private final Paint labelPaint;
    private final Paint lumenPaint;
    private final Paint outlinePaint;
    private volatile TractPosterior posterior;
    private final Paint uncertaintyPaint;

    /* JADX DEBUG: Multi-variable search result rejected for r0v2, resolved type: java.lang.Object[] */
    /* JADX WARN: 'this' call moved to the top of the method (can break code semantics) */
    /* JADX WARN: Multi-variable type inference failed */
    public TractSagittalView(Context context) {
        this(context, null, 2, 0 == true ? 1 : 0);
        Intrinsics.checkNotNullParameter(context, "context");
    }

    public /* synthetic */ TractSagittalView(Context context, AttributeSet attributeSet, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(context, (i & 2) != 0 ? null : attributeSet);
    }

    /* JADX WARN: 'super' call moved to the top of the method (can break code semantics) */
    public TractSagittalView(Context context, AttributeSet attributeSet) {
        super(context, attributeSet);
        Intrinsics.checkNotNullParameter(context, "context");
        Paint paint = new Paint(1);
        paint.setColor(Color.rgb(0, 108, 99));
        paint.setStyle(Paint.Style.FILL);
        this.lumenPaint = paint;
        Paint paint2 = new Paint(1);
        paint2.setColor(Color.argb(90, 0, 108, 99));
        paint2.setStyle(Paint.Style.FILL);
        this.uncertaintyPaint = paint2;
        Paint paint3 = new Paint(1);
        paint3.setColor(Color.rgb(0, 70, 64));
        paint3.setStrokeWidth(getResources().getDisplayMetrics().density * 2.0f);
        paint3.setStyle(Paint.Style.STROKE);
        this.outlinePaint = paint3;
        Paint paint4 = new Paint(1);
        paint4.setColor(Color.rgb(215, 229, 225));
        paint4.setStrokeWidth(getResources().getDisplayMetrics().density);
        paint4.setStyle(Paint.Style.STROKE);
        this.gridPaint = paint4;
        Paint paint5 = new Paint(1);
        paint5.setColor(Color.rgb(55, 67, 65));
        paint5.setTextSize(getResources().getDisplayMetrics().scaledDensity * 12.0f);
        this.labelPaint = paint5;
    }

    public final void updatePosterior(TractPosterior value) {
        TractPosterior copy;
        Intrinsics.checkNotNullParameter(value, "value");
        float[] sectionPosition = value.getSectionPosition();
        float[] copyOf = Arrays.copyOf(sectionPosition, sectionPosition.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        float[] areaCm2 = value.getAreaCm2();
        float[] copyOf2 = Arrays.copyOf(areaCm2, areaCm2.length);
        Intrinsics.checkNotNullExpressionValue(copyOf2, "copyOf(...)");
        float[] coefficients = value.getCoefficients();
        float[] copyOf3 = Arrays.copyOf(coefficients, coefficients.length);
        Intrinsics.checkNotNullExpressionValue(copyOf3, "copyOf(...)");
        copy = value.copy((r20 & 1) != 0 ? value.sectionPosition : copyOf, (r20 & 2) != 0 ? value.areaCm2 : copyOf2, (r20 & 4) != 0 ? value.coefficients : copyOf3, (r20 & 8) != 0 ? value.confidence : 0.0f, (r20 & 16) != 0 ? value.relativeAreaStd : 0.0f, (r20 & 32) != 0 ? value.interpretation : null, (r20 & 64) != 0 ? value.abstained : false, (r20 & Uuid.SIZE_BITS) != 0 ? value.abstentionReason : null, (r20 & 256) != 0 ? value.modeLabels : null);
        this.posterior = copy;
        postInvalidateOnAnimation();
    }

    @Override // android.view.View
    protected void onMeasure(int widthMeasureSpec, int heightMeasureSpec) {
        setMeasuredDimension(View.MeasureSpec.getSize(widthMeasureSpec), View.resolveSize((int) (getResources().getDisplayMetrics().density * 260.0f), heightMeasureSpec));
    }

    @Override // android.view.View
    protected void onDraw(Canvas canvas) {
        Intrinsics.checkNotNullParameter(canvas, "canvas");
        super.onDraw(canvas);
        canvas.drawColor(Color.rgb(248, 252, 250));
        RectF rectF = new RectF(getPaddingLeft() + 18.0f, getPaddingTop() + 24.0f, (getWidth() - getPaddingRight()) - 18.0f, (getHeight() - getPaddingBottom()) - 30.0f);
        for (int i = 0; i < 5; i++) {
            float width = rectF.left + ((rectF.width() * i) / 4.0f);
            canvas.drawLine(width, rectF.top, width, rectF.bottom, this.gridPaint);
        }
        TractPosterior tractPosterior = this.posterior;
        if (tractPosterior == null || tractPosterior.getAreaCm2().length == 0) {
            canvas.drawText("Waiting for live audio or offline replay", rectF.left, rectF.centerY(), this.labelPaint);
            return;
        }
        float[] radii = radii(tractPosterior, true);
        float[] radii2 = radii(tractPosterior, false);
        canvas.drawPath(lumenPath(rectF, tractPosterior.getSectionPosition(), radii), this.uncertaintyPaint);
        Path lumenPath = lumenPath(rectF, tractPosterior.getSectionPosition(), radii2);
        canvas.drawPath(lumenPath, this.lumenPaint);
        canvas.drawPath(lumenPath, this.outlinePaint);
        canvas.drawText("glottis", rectF.left, getHeight() - 8.0f, this.labelPaint);
        canvas.drawText("lips", rectF.right - this.labelPaint.measureText("lips"), getHeight() - 8.0f, this.labelPaint);
        canvas.drawText("estimated lumen • confidence " + ((int) (tractPosterior.getConfidence() * 100)) + "%", rectF.left, 17.0f, this.labelPaint);
    }

    private final float[] radii(TractPosterior state, boolean includeUncertainty) {
        Float maxOrNull = ArraysKt.maxOrNull(state.getAreaCm2());
        float max = Math.max(0.01f, maxOrNull != null ? maxOrNull.floatValue() : 1.0f);
        float relativeAreaStd = includeUncertainty ? 1.0f + state.getRelativeAreaStd() : 1.0f;
        int length = state.getAreaCm2().length;
        float[] fArr = new float[length];
        for (int i = 0; i < length; i++) {
            fArr[i] = RangesKt.coerceIn((float) Math.sqrt((state.getAreaCm2()[i] * relativeAreaStd) / max), 0.05f, 1.25f);
        }
        return fArr;
    }

    private final Path lumenPath(RectF bounds, float[] sectionPosition, float[] normalizedRadius) {
        int length = sectionPosition.length;
        float[] fArr = new float[length];
        float[] fArr2 = new float[sectionPosition.length];
        float[] fArr3 = new float[sectionPosition.length];
        int length2 = sectionPosition.length;
        for (int i = 0; i < length2; i++) {
            float f = sectionPosition[i];
            fArr[i] = bounds.left + (bounds.width() * f);
            fArr2[i] = bounds.bottom - (bounds.height() * ((((float) Math.sin((f * 3.141592653589793d) / 2.0f)) * 0.54f) + 0.18f));
            fArr3[i] = normalizedRadius[i] * bounds.height() * 0.13f;
        }
        Path path = new Path();
        path.moveTo(fArr[0], fArr2[0] - fArr3[0]);
        for (int i2 = 1; i2 < length; i2++) {
            path.lineTo(fArr[i2], fArr2[i2] - fArr3[i2]);
        }
        for (int lastIndex = ArraysKt.getLastIndex(fArr); -1 < lastIndex; lastIndex--) {
            path.lineTo(fArr[lastIndex], fArr2[lastIndex] + fArr3[lastIndex]);
        }
        path.close();
        return path;
    }
}

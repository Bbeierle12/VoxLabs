package org.vocaltract.pixel;

import android.app.ActivityManager;
import android.content.Context;
import android.opengl.GLSurfaceView;
import android.util.AttributeSet;
import android.view.MotionEvent;
import android.view.ScaleGestureDetector;
import android.view.ViewParent;
import kotlin.Metadata;
import kotlin.Unit;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: Tract3DView.kt */
@Metadata(d1 = {"\u0000h\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0002\n\u0002\u0010\u000b\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\u0010\u000e\n\u0002\u0010\u0002\n\u0002\b\t\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u000b\n\u0002\u0018\u0002\n\u0002\b\u0002\u0018\u0000 52\u00020\u0001:\u00015B\u001d\b\u0007\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\n\b\u0002\u0010\u0004\u001a\u0004\u0018\u00010\u0005¢\u0006\u0004\b\u0006\u0010\u0007J\u000e\u0010 \u001a\u00020\u00182\u0006\u0010!\u001a\u00020\"J\u000e\u0010#\u001a\u00020\u00182\u0006\u0010$\u001a\u00020%J\u000e\u0010&\u001a\u00020\u00142\u0006\u0010$\u001a\u00020'J\u0006\u0010(\u001a\u00020\u0014J\u0006\u0010)\u001a\u00020\u0018J\u000e\u0010*\u001a\u00020\u00182\u0006\u0010+\u001a\u00020\u0010J\u0006\u0010,\u001a\u00020\u0010J\u0006\u0010-\u001a\u00020\u0010J\u000e\u0010.\u001a\u00020\u00182\u0006\u0010+\u001a\u00020\u0010J\u0006\u0010/\u001a\u00020\u0010J\u0006\u00100\u001a\u00020\u0010J\u0010\u00101\u001a\u00020\u00102\u0006\u00102\u001a\u000203H\u0016J\b\u00104\u001a\u00020\u0010H\u0016R\u000e\u0010\b\u001a\u00020\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\n\u001a\u00020\u000bX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\f\u001a\u00020\rX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u000e\u001a\u00020\rX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u000f\u001a\u00020\u0010X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0011\u001a\u00020\u0010X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0012\u001a\u00020\u0010X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0013\u001a\u00020\u0014X\u0082\u000e¢\u0006\u0002\n\u0000R(\u0010\u0015\u001a\u0010\u0012\u0004\u0012\u00020\u0017\u0012\u0004\u0012\u00020\u0018\u0018\u00010\u0016X\u0086\u000e¢\u0006\u000e\n\u0000\u001a\u0004\b\u0019\u0010\u001a\"\u0004\b\u001b\u0010\u001cR(\u0010\u001d\u001a\u0010\u0012\u0004\u0012\u00020\u0014\u0012\u0004\u0012\u00020\u0018\u0018\u00010\u0016X\u0086\u000e¢\u0006\u000e\n\u0000\u001a\u0004\b\u001e\u0010\u001a\"\u0004\b\u001f\u0010\u001c"}, d2 = {"Lorg/vocaltract/pixel/Tract3DView;", "Landroid/opengl/GLSurfaceView;", "context", "Landroid/content/Context;", "attrs", "Landroid/util/AttributeSet;", "<init>", "(Landroid/content/Context;Landroid/util/AttributeSet;)V", "tractRenderer", "Lorg/vocaltract/pixel/TractMeshRenderer;", "scaleDetector", "Landroid/view/ScaleGestureDetector;", "lastTouchX", "", "lastTouchY", "moved", "", "cutaway", "wireframe", "overlaySummary", "Lorg/vocaltract/pixel/AcousticMatchSummary;", "onRendererFailure", "Lkotlin/Function1;", "", "", "getOnRendererFailure", "()Lkotlin/jvm/functions/Function1;", "setOnRendererFailure", "(Lkotlin/jvm/functions/Function1;)V", "onAcousticOverlayChanged", "getOnAcousticOverlayChanged", "setOnAcousticOverlayChanged", "setMeshAsset", "asset", "Lorg/vocaltract/pixel/TractLumenAsset;", "updatePosterior", "value", "Lorg/vocaltract/pixel/TractPosterior;", "updateAcoustics", "Lorg/vocaltract/pixel/AcousticEstimate;", "currentOverlaySummary", "resetView", "setCutawayEnabled", "enabled", "toggleCutaway", "isCutawayEnabled", "setWireframeEnabled", "toggleWireframe", "isWireframeEnabled", "onTouchEvent", "event", "Landroid/view/MotionEvent;", "performClick", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class Tract3DView extends GLSurfaceView {

    /* renamed from: Companion, reason: from kotlin metadata */
    public static final Companion INSTANCE = new Companion(null);
    private boolean cutaway;
    private float lastTouchX;
    private float lastTouchY;
    private boolean moved;
    private Function1<? super AcousticMatchSummary, Unit> onAcousticOverlayChanged;
    private Function1<? super String, Unit> onRendererFailure;
    private volatile AcousticMatchSummary overlaySummary;
    private final ScaleGestureDetector scaleDetector;
    private final TractMeshRenderer tractRenderer;
    private boolean wireframe;

    /* JADX DEBUG: Multi-variable search result rejected for r0v2, resolved type: java.lang.Object[] */
    /* JADX WARN: 'this' call moved to the top of the method (can break code semantics) */
    /* JADX WARN: Multi-variable type inference failed */
    public Tract3DView(Context context) {
        this(context, null, 2, 0 == true ? 1 : 0);
        Intrinsics.checkNotNullParameter(context, "context");
    }

    public /* synthetic */ Tract3DView(Context context, AttributeSet attributeSet, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(context, (i & 2) != 0 ? null : attributeSet);
    }

    /* JADX WARN: 'super' call moved to the top of the method (can break code semantics) */
    public Tract3DView(Context context, AttributeSet attributeSet) {
        super(context, attributeSet);
        Intrinsics.checkNotNullParameter(context, "context");
        TractMeshRenderer tractMeshRenderer = new TractMeshRenderer(new Function1() { // from class: org.vocaltract.pixel.Tract3DView$$ExternalSyntheticLambda0
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function1
            public final Object invoke(Object obj) {
                Unit tractRenderer$lambda$1;
                tractRenderer$lambda$1 = Tract3DView.tractRenderer$lambda$1(Tract3DView.this, (String) obj);
                return tractRenderer$lambda$1;
            }
        });
        this.tractRenderer = tractMeshRenderer;
        this.scaleDetector = new ScaleGestureDetector(context, new Tract3DView$scaleDetector$1(this));
        this.overlaySummary = AcousticMatchSummary.INSTANCE.getNONE();
        setEGLContextClientVersion(3);
        setEGLConfigChooser(8, 8, 8, 8, 24, 0);
        setRenderer(tractMeshRenderer);
        setRenderMode(1);
        setPreserveEGLContextOnPause(true);
        setContentDescription("Interactive 3D estimated vocal-tract lumen; drag to rotate and pinch to zoom");
        setClickable(true);
        setFocusable(true);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit tractRenderer$lambda$1(final Tract3DView tract3DView, final String message) {
        Intrinsics.checkNotNullParameter(message, "message");
        tract3DView.post(new Runnable() { // from class: org.vocaltract.pixel.Tract3DView$$ExternalSyntheticLambda5
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Runnable
            public final void run() {
                Tract3DView.tractRenderer$lambda$1$lambda$0(Tract3DView.this, message);
            }
        });
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void tractRenderer$lambda$1$lambda$0(Tract3DView tract3DView, String str) {
        Function1<? super String, Unit> function1 = tract3DView.onRendererFailure;
        if (function1 != null) {
            function1.invoke(str);
        }
    }

    /* JADX DEBUG: Type inference failed for r0v1. Raw type applied. Possible types: kotlin.jvm.functions.Function1<? super java.lang.String, kotlin.Unit>, kotlin.jvm.functions.Function1<java.lang.String, kotlin.Unit> */
    public final Function1<String, Unit> getOnRendererFailure() {
        return this.onRendererFailure;
    }

    public final void setOnRendererFailure(Function1<? super String, Unit> function1) {
        this.onRendererFailure = function1;
    }

    /* JADX DEBUG: Type inference failed for r0v1. Raw type applied. Possible types: kotlin.jvm.functions.Function1<? super org.vocaltract.pixel.AcousticMatchSummary, kotlin.Unit>, kotlin.jvm.functions.Function1<org.vocaltract.pixel.AcousticMatchSummary, kotlin.Unit> */
    public final Function1<AcousticMatchSummary, Unit> getOnAcousticOverlayChanged() {
        return this.onAcousticOverlayChanged;
    }

    public final void setOnAcousticOverlayChanged(Function1<? super AcousticMatchSummary, Unit> function1) {
        this.onAcousticOverlayChanged = function1;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void setMeshAsset$lambda$2(Tract3DView tract3DView, TractLumenAsset tractLumenAsset) {
        tract3DView.tractRenderer.setMeshAsset(tractLumenAsset);
    }

    public final void setMeshAsset(final TractLumenAsset asset) {
        Intrinsics.checkNotNullParameter(asset, "asset");
        queueEvent(new Runnable() { // from class: org.vocaltract.pixel.Tract3DView$$ExternalSyntheticLambda6
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Runnable
            public final void run() {
                Tract3DView.setMeshAsset$lambda$2(Tract3DView.this, asset);
            }
        });
    }

    public final void updatePosterior(TractPosterior value) {
        Intrinsics.checkNotNullParameter(value, "value");
        this.tractRenderer.submitPosterior(value);
    }

    public final AcousticMatchSummary updateAcoustics(AcousticEstimate value) {
        Intrinsics.checkNotNullParameter(value, "value");
        AcousticMatchSummary compute = AcousticOverlayMath.INSTANCE.compute(value);
        this.overlaySummary = compute;
        this.tractRenderer.submitOverlay(compute);
        Function1<? super AcousticMatchSummary, Unit> function1 = this.onAcousticOverlayChanged;
        if (function1 != null) {
            function1.invoke(compute);
        }
        return compute;
    }

    /* renamed from: currentOverlaySummary, reason: from getter */
    public final AcousticMatchSummary getOverlaySummary() {
        return this.overlaySummary;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void resetView$lambda$3(Tract3DView tract3DView) {
        tract3DView.tractRenderer.resetCamera();
    }

    public final void resetView() {
        queueEvent(new Runnable() { // from class: org.vocaltract.pixel.Tract3DView$$ExternalSyntheticLambda1
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Runnable
            public final void run() {
                Tract3DView.resetView$lambda$3(Tract3DView.this);
            }
        });
    }

    public final void setCutawayEnabled(final boolean enabled) {
        this.cutaway = enabled;
        queueEvent(new Runnable() { // from class: org.vocaltract.pixel.Tract3DView$$ExternalSyntheticLambda4
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Runnable
            public final void run() {
                Tract3DView.setCutawayEnabled$lambda$4(Tract3DView.this, enabled);
            }
        });
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void setCutawayEnabled$lambda$4(Tract3DView tract3DView, boolean z) {
        tract3DView.tractRenderer.setCutawayEnabled(z);
    }

    public final boolean toggleCutaway() {
        setCutawayEnabled(!this.cutaway);
        return this.cutaway;
    }

    /* renamed from: isCutawayEnabled, reason: from getter */
    public final boolean getCutaway() {
        return this.cutaway;
    }

    public final void setWireframeEnabled(final boolean enabled) {
        this.wireframe = enabled;
        queueEvent(new Runnable() { // from class: org.vocaltract.pixel.Tract3DView$$ExternalSyntheticLambda2
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Runnable
            public final void run() {
                Tract3DView.setWireframeEnabled$lambda$5(Tract3DView.this, enabled);
            }
        });
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void setWireframeEnabled$lambda$5(Tract3DView tract3DView, boolean z) {
        tract3DView.tractRenderer.setWireframeEnabled(z);
    }

    public final boolean toggleWireframe() {
        setWireframeEnabled(!this.wireframe);
        return this.wireframe;
    }

    /* renamed from: isWireframeEnabled, reason: from getter */
    public final boolean getWireframe() {
        return this.wireframe;
    }

    @Override // android.view.View
    public boolean onTouchEvent(MotionEvent event) {
        ViewParent parent;
        Intrinsics.checkNotNullParameter(event, "event");
        this.scaleDetector.onTouchEvent(event);
        int actionMasked = event.getActionMasked();
        if (actionMasked == 0) {
            this.lastTouchX = event.getX();
            this.lastTouchY = event.getY();
            this.moved = false;
            ViewParent parent2 = getParent();
            if (parent2 != null) {
                parent2.requestDisallowInterceptTouchEvent(true);
            }
            return true;
        }
        if (actionMasked == 1) {
            ViewParent parent3 = getParent();
            if (parent3 != null) {
                parent3.requestDisallowInterceptTouchEvent(false);
            }
            if (!this.moved) {
                performClick();
            }
            return true;
        }
        if (actionMasked != 2) {
            if (actionMasked == 3 && (parent = getParent()) != null) {
                parent.requestDisallowInterceptTouchEvent(false);
            }
            return true;
        }
        final float x = event.getX() - this.lastTouchX;
        final float y = event.getY() - this.lastTouchY;
        this.lastTouchX = event.getX();
        this.lastTouchY = event.getY();
        if (!this.scaleDetector.isInProgress() && (Math.abs(x) > 0.0f || Math.abs(y) > 0.0f)) {
            final float coerceAtLeast = RangesKt.coerceAtLeast(getResources().getDisplayMetrics().density, 1.0f);
            queueEvent(new Runnable() { // from class: org.vocaltract.pixel.Tract3DView$$ExternalSyntheticLambda3
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // java.lang.Runnable
                public final void run() {
                    Tract3DView.onTouchEvent$lambda$6(Tract3DView.this, y, coerceAtLeast, x);
                }
            });
            this.moved = true;
        }
        return true;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void onTouchEvent$lambda$6(Tract3DView tract3DView, float f, float f2, float f3) {
        tract3DView.tractRenderer.rotateBy((f * 0.24f) / f2, (f3 * 0.24f) / f2);
    }

    @Override // android.view.View
    public boolean performClick() {
        super.performClick();
        return true;
    }

    /* compiled from: Tract3DView.kt */
    @Metadata(d1 = {"\u0000\u0016\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\u000b\n\u0000\n\u0002\u0018\u0002\b\u0086\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u000e\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u0007"}, d2 = {"Lorg/vocaltract/pixel/Tract3DView$Companion;", "", "<init>", "()V", "supportsGles3", "", "context", "Landroid/content/Context;"}, k = 1, mv = {2, 0, 0}, xi = 48)
    public static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }

        public final boolean supportsGles3(Context context) {
            Intrinsics.checkNotNullParameter(context, "context");
            Object systemService = context.getSystemService("activity");
            ActivityManager activityManager = systemService instanceof ActivityManager ? (ActivityManager) systemService : null;
            return activityManager != null && activityManager.getDeviceConfigurationInfo().reqGlEsVersion >= 196608;
        }
    }
}

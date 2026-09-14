package org.vocaltract.pixel;

import android.view.ScaleGestureDetector;
import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: Tract3DView.kt */
@Metadata(d1 = {"\u0000\u0015\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u000b\n\u0000\n\u0002\u0018\u0002*\u0001\u0000\b\n\u0018\u00002\u00020\u0001J\u0010\u0010\u0002\u001a\u00020\u00032\u0006\u0010\u0004\u001a\u00020\u0005H\u0016"}, d2 = {"org/vocaltract/pixel/Tract3DView$scaleDetector$1", "Landroid/view/ScaleGestureDetector$SimpleOnScaleGestureListener;", "onScale", "", "detector", "Landroid/view/ScaleGestureDetector;"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class Tract3DView$scaleDetector$1 extends ScaleGestureDetector.SimpleOnScaleGestureListener {
    final /* synthetic */ Tract3DView this$0;

    Tract3DView$scaleDetector$1(Tract3DView tract3DView) {
        this.this$0 = tract3DView;
    }

    @Override // android.view.ScaleGestureDetector.SimpleOnScaleGestureListener, android.view.ScaleGestureDetector.OnScaleGestureListener
    public boolean onScale(ScaleGestureDetector detector) {
        Intrinsics.checkNotNullParameter(detector, "detector");
        final float scaleFactor = detector.getScaleFactor();
        if (Float.isInfinite(scaleFactor) || Float.isNaN(scaleFactor) || scaleFactor <= 0.0f) {
            return true;
        }
        final Tract3DView tract3DView = this.this$0;
        tract3DView.queueEvent(new Runnable() { // from class: org.vocaltract.pixel.Tract3DView$scaleDetector$1$$ExternalSyntheticLambda0
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Runnable
            public final void run() {
                Tract3DView$scaleDetector$1.onScale$lambda$0(Tract3DView.this, scaleFactor);
            }
        });
        return true;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void onScale$lambda$0(Tract3DView tract3DView, float f) {
        TractMeshRenderer tractMeshRenderer;
        tractMeshRenderer = tract3DView.tractRenderer;
        tractMeshRenderer.zoomBy(f);
    }
}

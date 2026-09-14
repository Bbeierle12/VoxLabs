package org.vocaltract.pixel;

import android.graphics.Bitmap;
import android.webkit.RenderProcessGoneDetail;
import android.webkit.WebResourceRequest;
import android.webkit.WebResourceResponse;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.Toast;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import kotlin.Metadata;
import kotlin.Pair;
import kotlin.TuplesKt;
import kotlin.Unit;
import kotlin.collections.MapsKt;
import kotlin.jvm.functions.Function0;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: AnatomyBenchActivity.kt */
@Metadata(d1 = {"\u0000;\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u000b\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0002\n\u0000\n\u0002\u0010\u000e\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002*\u0001\u0000\b\n\u0018\u00002\u00020\u0001J\u0018\u0010\u0002\u001a\u00020\u00032\u0006\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u0007H\u0016J$\u0010\b\u001a\u00020\t2\u0006\u0010\u0004\u001a\u00020\u00052\b\u0010\n\u001a\u0004\u0018\u00010\u000b2\b\u0010\f\u001a\u0004\u0018\u00010\rH\u0016J\u001a\u0010\u000e\u001a\u00020\t2\u0006\u0010\u0004\u001a\u00020\u00052\b\u0010\n\u001a\u0004\u0018\u00010\u000bH\u0016J\u0018\u0010\u000f\u001a\u00020\u00032\u0006\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0010\u001a\u00020\u0011H\u0016J\u0018\u0010\u0012\u001a\u00020\u00132\u0006\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u0007H\u0016"}, d2 = {"org/vocaltract/pixel/AnatomyBenchActivity$buildUi$3", "Landroid/webkit/WebViewClient;", "shouldOverrideUrlLoading", "", "view", "Landroid/webkit/WebView;", "request", "Landroid/webkit/WebResourceRequest;", "onPageStarted", "", "url", "", "favicon", "Landroid/graphics/Bitmap;", "onPageFinished", "onRenderProcessGone", "detail", "Landroid/webkit/RenderProcessGoneDetail;", "shouldInterceptRequest", "Landroid/webkit/WebResourceResponse;"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class AnatomyBenchActivity$buildUi$3 extends WebViewClient {
    final /* synthetic */ AnatomyBenchActivity this$0;

    @Override // android.webkit.WebViewClient
    public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest request) {
        Intrinsics.checkNotNullParameter(view, "view");
        Intrinsics.checkNotNullParameter(request, "request");
        return true;
    }

    AnatomyBenchActivity$buildUi$3(AnatomyBenchActivity anatomyBenchActivity) {
        this.this$0 = anatomyBenchActivity;
    }

    @Override // android.webkit.WebViewClient
    public void onPageStarted(WebView view, String url, Bitmap favicon) {
        Intrinsics.checkNotNullParameter(view, "view");
        this.this$0.trusted = false;
        this.this$0.stopEverything();
    }

    @Override // android.webkit.WebViewClient
    public void onPageFinished(WebView view, String url) {
        Intrinsics.checkNotNullParameter(view, "view");
        this.this$0.trusted = Intrinsics.areEqual(url, LabAssets.PAGE);
        if (this.this$0.trusted) {
            AnatomyBenchActivity.submit$default(this.this$0, null, null, new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$buildUi$3$$ExternalSyntheticLambda0
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function0
                public final Object invoke() {
                    Unit unit;
                    unit = Unit.INSTANCE;
                    return unit;
                }
            }, 3, null);
        }
    }

    @Override // android.webkit.WebViewClient
    public boolean onRenderProcessGone(WebView view, RenderProcessGoneDetail detail) {
        Intrinsics.checkNotNullParameter(view, "view");
        Intrinsics.checkNotNullParameter(detail, "detail");
        this.this$0.trusted = false;
        this.this$0.stopEverything();
        Toast.makeText(this.this$0, "Lab renderer stopped; audio stopped. Reopen the app.", 1).show();
        this.this$0.finish();
        return true;
    }

    @Override // android.webkit.WebViewClient
    public WebResourceResponse shouldInterceptRequest(WebView view, WebResourceRequest request) {
        Intrinsics.checkNotNullParameter(view, "view");
        Intrinsics.checkNotNullParameter(request, "request");
        LabAssets labAssets = LabAssets.INSTANCE;
        String uri = request.getUrl().toString();
        Intrinsics.checkNotNullExpressionValue(uri, "toString(...)");
        Pair<String, String> resolve = labAssets.resolve(uri);
        if (!Intrinsics.areEqual(request.getMethod(), "GET") || resolve == null) {
            return new WebResourceResponse("text/plain", "UTF-8", 403, "Blocked", MapsKt.emptyMap(), new ByteArrayInputStream(new byte[0]));
        }
        try {
            return new WebResourceResponse(resolve.getSecond(), "UTF-8", 200, "OK", MapsKt.mapOf(TuplesKt.to("X-Content-Type-Options", "nosniff"), TuplesKt.to("Cache-Control", "no-store")), this.this$0.getAssets().open(resolve.getFirst()));
        } catch (IOException unused) {
            return new WebResourceResponse("text/plain", "UTF-8", 404, "Missing", MapsKt.emptyMap(), new ByteArrayInputStream(new byte[0]));
        }
    }
}

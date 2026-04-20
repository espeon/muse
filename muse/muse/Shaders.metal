#include <metal_stdlib>
#include <SwiftUI/SwiftUI_Metal.h>

using namespace metal;

[[ stitchable ]] half4 dreamySpiral(
    float2 position,
    SwiftUI::Layer layer,
    float time,
    float2 size
) {
    int layers = 5;
    float2 baseCenter = size * 0.5;
    half4 accumulated = half4(0.0);

    for (int i = 0; i < layers; i++) {
        float fi = float(i);

        // calculate params for the layer
        float speed       = 0.08 + fi * 0.05;
        float rotSpeed    = 0.15 + fi * 0.12 * (i % 2 == 1 ? -1 : 1);
        float radius      = size.x * (0.05 + fi * 0.03);
        float zoom        = 1.6 + fi * 0.12;

        float2 localCenter =
            baseCenter +
            float2(
                cos(fi * 1.7),
                sin(fi * 2.3)
            ) * size * 0.12;

        // Time with individual speed
        float t = time * speed + fi * 2.1;

        // Spiral motion
        float2 spiralOffset =
            float2(cos(t), sin(t)) * radius;

        // Relative position
        float2 rel = position - localCenter + spiralOffset;

        // Independent rotation per layer
        float rot = time * rotSpeed + fi;
        float2 rotated = float2(
            rel.x * cos(rot) - rel.y * sin(rot),
            rel.x * sin(rot) + rel.y * cos(rot)
        );

        // Gentle breathing zoom
        float dynamicZoom =
            zoom + sin(time * (0.1 + fi * 0.03)) * 0.25;

        float2 samplePos =
            localCenter + rotated / dynamicZoom;


        accumulated += layer.sample(samplePos);
    }

    return accumulated / float(layers);
}

[[ stitchable ]] float2 marquee(float2 position, float time, float width, float speed, float spacing) {
    float newX = fmod(position.x + time * 20 * speed, width + spacing);

    return float2(newX, position.y);
}

[[ stitchable ]] float2 karaokeWave(float2 position, float2 size, float sweepProgress, float waveAmplitude) {
    if (waveAmplitude <= 0.0) {
        return position;
    }

    float2 uv = position / size;
    float waveX = (uv.x - sweepProgress) * 10.0;
    float wave = sin(waveX) * waveAmplitude;
    wave *= smoothstep(0.0, 0.3, sweepProgress) * smoothstep(1.0, 0.7, uv.x);

    return float2(position.x, position.y + wave);
}

[[ stitchable ]] half4 karaokeSweep(float2 position, half4 color, float2 size, float sweep, float edgeWidthPixels) {
    float uv_x = position.x / size.x;
    float gradientWidth = (edgeWidthPixels / size.x) * 2.0;

    float sweepEdge = smoothstep(sweep - gradientWidth, sweep + gradientWidth, uv_x);
    return color * mix(1.0, 0.5, sweepEdge);
}

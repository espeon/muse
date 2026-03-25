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

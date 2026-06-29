import { useEffect, useRef } from "react";
import { cn } from "@/lib/utils";

/** Real WebGL 2 fragment shader port of the iOS dreamySpiral Metal shader.
 *  Samples the artist image 5 times with per-layer rotation, spiral motion,
 *  zoom and breathing, then blends the result. No Three.js needed.
 */
export function ShaderBackground({
  image,
  className,
  children,
}: {
  image?: string;
  className?: string;
  children?: React.ReactNode;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const imageRef = useRef<HTMLImageElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !image) return;

    const gl = canvas.getContext("webgl2", {
      alpha: true,
      antialias: false,
      powerPreference: "low-power",
    });
    if (!gl) return;

    // capture in consts so TS narrows for nested callbacks
    const g = gl;
    const c = canvas;

    const vertexSrc = `#version 300 es
    precision mediump float;
    in vec2 a_position;
    out vec2 v_uv;
    void main() {
      v_uv = a_position * 0.5 + 0.5;
      gl_Position = vec4(a_position, 0.0, 1.0);
    }`;

    const fragmentSrc = `#version 300 es
    precision mediump float;

    in vec2 v_uv;
    out vec4 outColor;

    uniform sampler2D u_texture;
    uniform vec2 u_size;
    uniform float u_time;
    uniform float u_time_scale;

    void main() {
      int layers = 5;
      vec2 position = gl_FragCoord.xy;
      vec2 baseCenter = u_size * 0.5;
      vec4 accumulated = vec4(0.0);

      float t = u_time * u_time_scale;

      for (int i = 0; i < layers; i++) {
        float fi = float(i);

        float speed = 0.08 + fi * 0.05;
        float rotSpeed = 0.15 + fi * 0.12 * (mod(float(i), 2.0) == 1.0 ? -1.0 : 1.0);
        float radius = u_size.x * (0.05 + fi * 0.03);
        float zoom = 1.6 + fi * 0.12;

        vec2 localCenter = baseCenter + vec2(cos(fi * 1.7), sin(fi * 2.3)) * u_size * 0.12;

        float t_layer = t * speed + fi * 2.1;
        vec2 spiralOffset = vec2(cos(t_layer), sin(t_layer)) * radius;

        vec2 rel = position - localCenter + spiralOffset;

        float rot = t * rotSpeed + fi;
        vec2 rotated = vec2(
          rel.x * cos(rot) - rel.y * sin(rot),
          rel.x * sin(rot) + rel.y * cos(rot)
        );

        float dynamicZoom = zoom + sin(t * (0.1 + fi * 0.03)) * 0.25;

        vec2 samplePos = (localCenter + rotated / dynamicZoom) / u_size;
        accumulated += texture(u_texture, samplePos);
      }

      vec4 color = accumulated / float(layers);

      // vignette + darken so text stays readable
      vec2 center = (position / u_size) - 0.5;
      float vignette = 1.0 - dot(center, center) * 0.6;
      color.rgb *= vignette * 0.7;

      outColor = color;
    }`;

    function compile(type: number, src: string) {
      const shader = g.createShader(type);
      if (!shader) return null;
      g.shaderSource(shader, src);
      g.compileShader(shader);
      if (!g.getShaderParameter(shader, g.COMPILE_STATUS)) {
        console.error(g.getShaderInfoLog(shader));
        g.deleteShader(shader);
        return null;
      }
      return shader;
    }

    const vs = compile(g.VERTEX_SHADER, vertexSrc);
    const fs = compile(g.FRAGMENT_SHADER, fragmentSrc);
    if (!vs || !fs) return;
    const program = g.createProgram();
    if (!program) return;
    g.attachShader(program, vs);
    g.attachShader(program, fs);
    g.linkProgram(program);
    if (!g.getProgramParameter(program, g.LINK_STATUS)) {
      console.error(g.getProgramInfoLog(program));
      return;
    }

    const aPosition = g.getAttribLocation(program, "a_position");
    const uTexture = g.getUniformLocation(program, "u_texture");
    const uSize = g.getUniformLocation(program, "u_size");
    const uTime = g.getUniformLocation(program, "u_time");
    const uTimeScale = g.getUniformLocation(program, "u_time_scale");

    const vbo = g.createBuffer();
    g.bindBuffer(g.ARRAY_BUFFER, vbo);
    g.bufferData(
      g.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
      g.STATIC_DRAW,
    );

    const texture = g.createTexture();
    g.bindTexture(g.TEXTURE_2D, texture);
    g.texParameteri(g.TEXTURE_2D, g.TEXTURE_WRAP_S, g.CLAMP_TO_EDGE);
    g.texParameteri(g.TEXTURE_2D, g.TEXTURE_WRAP_T, g.CLAMP_TO_EDGE);
    g.texParameteri(g.TEXTURE_2D, g.TEXTURE_MIN_FILTER, g.LINEAR);
    g.texParameteri(g.TEXTURE_2D, g.TEXTURE_MAG_FILTER, g.LINEAR);

    // 1x1 transparent placeholder until image loads
    g.texImage2D(
      g.TEXTURE_2D,
      0,
      g.RGBA,
      1,
      1,
      0,
      g.RGBA,
      g.UNSIGNED_BYTE,
      new Uint8Array([0, 0, 0, 0]),
    );

    let raf = 0;
    let start = performance.now();
    let disposed = false;

    function draw() {
      if (disposed) return;
      const w = c.clientWidth;
      const h = c.clientHeight;
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      if (c.width !== Math.floor(w * dpr) || c.height !== Math.floor(h * dpr)) {
        c.width = Math.floor(w * dpr);
        c.height = Math.floor(h * dpr);
        g.viewport(0, 0, c.width, c.height);
      }

      const time = (performance.now() - start) / 1000;

      g.useProgram(program);
      g.bindBuffer(g.ARRAY_BUFFER, vbo);
      g.enableVertexAttribArray(aPosition);
      g.vertexAttribPointer(aPosition, 2, g.FLOAT, false, 0, 0);

      g.activeTexture(g.TEXTURE0);
      g.bindTexture(g.TEXTURE_2D, texture);
      g.uniform1i(uTexture, 0);
      g.uniform2f(uSize, c.width, c.height);
      g.uniform1f(uTime, time);
      g.uniform1f(uTimeScale, 0.25);

      g.drawArrays(g.TRIANGLES, 0, 6);
      raf = requestAnimationFrame(draw);
    }

    // load image and upload texture
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => {
      imageRef.current = img;
      g.bindTexture(g.TEXTURE_2D, texture);
      g.texImage2D(g.TEXTURE_2D, 0, g.RGBA, g.RGBA, g.UNSIGNED_BYTE, img);
    };
    img.src = image;

    raf = requestAnimationFrame(draw);

    const ro = new ResizeObserver(() => {
      // resize handled in draw loop via clientWidth/Height
    });
    ro.observe(c);

    return () => {
      disposed = true;
      cancelAnimationFrame(raf);
      ro.disconnect();
      g.deleteProgram(program);
      g.deleteShader(vs);
      g.deleteShader(fs);
      g.deleteBuffer(vbo);
      g.deleteTexture(texture);
    };
  }, [image]);

  if (!image) {
    return (
      <div className={cn("bg-gradient-to-br from-card via-background to-accent", className)}>
        {children}
      </div>
    );
  }

  return (
    <div className={cn("relative overflow-hidden", className)}>
      <canvas
        ref={canvasRef}
        className="absolute inset-0 h-full w-full object-cover"
        style={{ filter: "blur(24px)" }}
      />
      {children}
    </div>
  );
}

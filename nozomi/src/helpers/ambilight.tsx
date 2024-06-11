export default function Ambilight() {
  return (
    <svg width="0" height="0">
      <filter
        id="ambilight"
        width="800%"
        height="800%"
        x="-0.75"
        y="-0.75"
        colorInterpolationFilters="sRGB"
      >
        <feOffset in="SourceGraphic" result="source-copy"></feOffset>
        <feColorMatrix
          in="source-copy"
          type="saturate"
          values="1.5"
          result="saturated-copy"
        ></feColorMatrix>
        <feColorMatrix
          in="saturated-copy"
          type="matrix"
          values="1 0 0 0 0
                   0 1 0 0 0
                   0 0 1 0 0
                   33 33 33 101 -100"
          result="bright-colors"
        ></feColorMatrix>
        <feMorphology
          in="bright-colors"
          operator="dilate"
          radius="1"
          result="spread"
        ></feMorphology>
        <feGaussianBlur
          in="spread"
          stdDeviation="83"
          result="ambilight-light"
        ></feGaussianBlur>
        <feOffset in="SourceGraphic" result="source"></feOffset>
        <feComposite
          in="source"
          in2="ambilight-light"
          operator="over"
        ></feComposite>
      </filter>
    </svg>
  );
}

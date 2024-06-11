export function clamp(number: number, one: number, two: number) {
  let min = Math.min(one, two);
  let max = Math.max(one, two);
  return Math.max(min, Math.min(number, max));
}

export function mapPercent(x: number, min: number, max: number) {
  mapRange(x, min, max, 0, 100);
}

export function mapRange(
  x: number,
  in_min: number,
  in_max: number,
  out_min: number,
  out_max: number,
  easing: (e: number) => number = linear,
  doClamp: boolean = true
) {
  let res = (x - in_min) / (in_max - in_min);

  res = easing(res);

  res = out_min + (res * (out_max - out_min));

  if (doClamp) {
    res = clamp(res, out_min, out_max);
  }

  return res;
}

// Linear
export function linear(t: number) {
  return t;
}

// Quadratic
export function easeInQuad(t: number) {
  return t * t;
}

export function easeOutQuad(t: number) {
  return t * (2 - t);
}

export function easeInOutQuad(t: number) {
  return t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
}

// Cubic
export function easeInCubic(t: number) {
  return t * t * t;
}

export function easeOutCubic(t: number) {
  return --t * t * t + 1;
}

export function easeInOutCubic(t: number) {
  return t < 0.5 ? 4 * t * t * t : (t - 1) * (2 * t - 2) * (2 * t - 2) + 1;
}

// Quartic
export function easeInQuart(t: number) {
  return t * t * t * t;
}

export function easeOutQuart(t: number) {
  return 1 - --t * t * t * t;
}

export function easeInOutQuart(t: number) {
  return t < 0.5 ? 8 * t * t * t * t : 1 - 8 * --t * t * t * t;
}

// Quintic
export function easeInQuint(t: number) {
  return t * t * t * t * t;
}

export function easeOutQuint(t: number) {
  return 1 + --t * t * t * t * t;
}

export function easeInOutQuint(t: number) {
  return t < 0.5 ? 16 * t * t * t * t * t : 1 + 16 * --t * t * t * t * t;
}

// Sinusoidal
export function easeInSine(t: number) {
  return 1 - Math.cos((t * Math.PI) / 2);
}

export function easeOutSine(t: number) {
  return Math.sin((t * Math.PI) / 2);
}

export function easeInOutSine(t: number) {
  return -(Math.cos(Math.PI * t) - 1) / 2;
}

// Exponential
export function easeInExpo(t: number) {
  return t === 0 ? 0 : Math.pow(2, 10 * (t - 1));
}

export function easeOutExpo(t: number) {
  return t === 1 ? 1 : 1 - Math.pow(2, -10 * t);
}

export function easeInOutExpo(t: number) {
  if (t === 0) return 0;
  if (t === 1) return 1;
  if (t < 0.5) return Math.pow(2, 10 * (2 * t - 1)) / 2;
  return (2 - Math.pow(2, -10 * (2 * t - 1))) / 2;
}

// Circular
export function easeInCirc(t: number) {
  return 1 - Math.sqrt(1 - t * t);
}

export function easeOutCirc(t: number) {
  return Math.sqrt(1 - --t * t);
}

export function easeInOutCirc(t: number) {
  return t < 0.5
    ? (1 - Math.sqrt(1 - 2 * t * 2 * t)) / 2
    : (Math.sqrt(1 - 2 * --t * 2 * t) + 1) / 2;
}

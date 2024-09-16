"use client";
import React from "react";
import { EmblaOptionsType } from "embla-carousel";
import { DotButton, useDotButton } from "./carouselDots";
import { PrevButton, NextButton, usePrevNextButtons } from "./carouselArrows";
import useEmblaCarousel from "embla-carousel-react";
import { WheelGesturesPlugin } from "embla-carousel-wheel-gestures";
import { clsx } from "clsx";

type PropType = {
  //slides: number[];
  options?: EmblaOptionsType;
  children: React.ReactNode;
  className?: string;
};

const EmblaCarousel: React.FC<PropType> = (props) => {
  const { options, children } = props;
  const [emblaRef, emblaApi] = useEmblaCarousel(options, [
    WheelGesturesPlugin(),
  ]);

  const { selectedIndex, scrollSnaps, onDotButtonClick } =
    useDotButton(emblaApi);

  const {
    prevBtnDisabled,
    nextBtnDisabled,
    onPrevButtonClick,
    onNextButtonClick,
  } = usePrevNextButtons(emblaApi);

  return (
    <section
      className={clsx(
        "relative w-full overflow-visible -ml-2",
        props.className,
      )}
    >
      <div
        className={`h-full absolute -left-14 w-10 z-10 transition-all duration-1000 bg-gradient-to-l from-transparent via-slate-950 to-slate-950 ${prevBtnDisabled ? "opacity-0" : "opacity-100"}`}
      />
      <div
        className={`h-full absolute -right-14 w-20 z-10 transition-all duration-1000 bg-gradient-to-r from-transparent to-slate-950 ${nextBtnDisabled ? "opacity-0" : "opacity-100"}`}
      />
      <div className="w-[calc(100%-5rem)] " ref={emblaRef}>
        <div className="w-full flex touch-pinch-zoom touch-pan-y gap-x-2">
          {children}
        </div>
      </div>

      <div className=" grid-cols-2 hidden md:grid">
        <div className="space-x-1 mt-1">
          <PrevButton onClick={onPrevButtonClick} disabled={prevBtnDisabled} />
          <NextButton onClick={onNextButtonClick} disabled={nextBtnDisabled} />
        </div>

        <div className="w-full flex flex-row flex-wrap flex-end justify-end items-center space-x-2 z-20">
          {scrollSnaps.map((_, index) => (
            <DotButton
              key={index}
              onClick={() => onDotButtonClick(index)}
              className={"w-2 h-2 rounded-full border ".concat(
                index === selectedIndex ? "border-white" : "border-slate-700",
              )}
            />
          ))}
        </div>
      </div>
    </section>
  );
};

export function CarouselPage({
  key,
  children,
  className,
}: {
  key: number;
  children: React.ReactNode | string;
  className?: string;
}) {
  return (
    <div
      className={clsx(
        "min-w-0 flex-[0_0_calc(100%/2-0.5rem)] lg:flex-[0_0_calc(100%/4-1rem)] 2xl:flex-[0_0_calc(100%/5-1rem)]",
        className,
      )}
      key={key}
    >
      {children}
    </div>
  );
}

export default EmblaCarousel;

"use client";
import React from "react";
import { EmblaOptionsType } from "embla-carousel";
import { DotButton, useDotButton } from "./carouselDots";
import { PrevButton, NextButton, usePrevNextButtons } from "./carouselArrows";
import useEmblaCarousel from "embla-carousel-react";

type PropType = {
  //slides: number[];
  options?: EmblaOptionsType;
  children: React.ReactNode;
};

const EmblaCarousel: React.FC<PropType> = (props) => {
  const { options, children } = props;
  const [emblaRef, emblaApi] = useEmblaCarousel(options);

  const { selectedIndex, scrollSnaps, onDotButtonClick } =
    useDotButton(emblaApi);

  const {
    prevBtnDisabled,
    nextBtnDisabled,
    onPrevButtonClick,
    onNextButtonClick,
  } = usePrevNextButtons(emblaApi);

  return (
    <section className="w-full overflow-hidden -ml-2">
      <div className="w-full " ref={emblaRef}>
        <div className="w-full flex touch-pinch-zoom touch-pan-y gap-x-2">
          {children}
        </div>
      </div>

      <div className=" grid-cols-2 ml-2 hidden md:grid">
        <div className="space-x-1 mt-1">
          <PrevButton onClick={onPrevButtonClick} disabled={prevBtnDisabled} />
          <NextButton onClick={onNextButtonClick} disabled={nextBtnDisabled} />
        </div>

        <div className="w-full flex flex-row flex-wrap flex-end justify-end items-center space-x-2">
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
}: {
  key: number;
  children: React.ReactNode;
}) {
  return (
    <div
      className="min-w-0 flex-[0_0_calc(100%/3-1rem)] lg:flex-[0_0_calc(100%/5-1rem)] 2xl:flex-[0_0_calc(100%/7-1rem)]"
      key={key}
    >
      {children}
    </div>
  );
}

export default EmblaCarousel;

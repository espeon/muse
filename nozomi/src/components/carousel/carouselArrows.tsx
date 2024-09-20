import React, {
  ComponentPropsWithRef,
  useCallback,
  useEffect,
  useState,
} from "react";
import { EmblaCarouselType } from "embla-carousel";
import {
  LuArrowLeft,
  LuArrowRight,
  LuChevronLeft,
  LuChevronRight,
} from "react-icons/lu";
import clsx from "clsx";

type UsePrevNextButtonsType = {
  prevBtnDisabled: boolean;
  nextBtnDisabled: boolean;
  onPrevButtonClick: () => void;
  onNextButtonClick: () => void;
};

export const usePrevNextButtons = (
  emblaApi: EmblaCarouselType | undefined,
): UsePrevNextButtonsType => {
  const [prevBtnDisabled, setPrevBtnDisabled] = useState(true);
  const [nextBtnDisabled, setNextBtnDisabled] = useState(true);

  const onPrevButtonClick = useCallback(() => {
    if (!emblaApi) return;
    emblaApi.scrollPrev();
  }, [emblaApi]);

  const onNextButtonClick = useCallback(() => {
    if (!emblaApi) return;
    emblaApi.scrollNext();
  }, [emblaApi]);

  const onSelect = useCallback((emblaApi: EmblaCarouselType) => {
    setPrevBtnDisabled(!emblaApi.canScrollPrev());
    setNextBtnDisabled(!emblaApi.canScrollNext());
  }, []);

  useEffect(() => {
    if (!emblaApi) return;

    onSelect(emblaApi);
    emblaApi.on("reInit", onSelect).on("select", onSelect);
  }, [emblaApi, onSelect]);

  return {
    prevBtnDisabled,
    nextBtnDisabled,
    onPrevButtonClick,
    onNextButtonClick,
  };
};

type PropType = ComponentPropsWithRef<"button">;

export const PrevButton: React.FC<PropType> = (props) => {
  const { children, ...restProps } = props;

  return (
    <button
      className="p-1 rounded-full bg-slate-700 disabled:bg-slate-700/50 transition-all duration-150"
      disabled={props.disabled}
      {...restProps}
    >
      <LuChevronLeft
        className={clsx(
          props.disabled && "text-slate-400",
          "pr-0.5 w-6 h-6 text-white transition-all duration-150",
        )}
      />
    </button>
  );
};

export const NextButton: React.FC<PropType> = (props) => {
  const { children, ...restProps } = props;

  return (
    <button
      className="p-1 rounded-full bg-slate-700 disabled:bg-slate-700/50 transition-all duration-150"
      disabled={props.disabled}
      {...restProps}
    >
      <LuChevronRight
        className={clsx(
          props.disabled && "text-slate-400",
          "pl-0.5 w-6 h-6 text-white transition-all duration-150",
        )}
      />
      {children}
    </button>
  );
};

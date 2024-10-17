import React from "react";
import * as Select from "@radix-ui/react-select";
import * as Toggle from "@radix-ui/react-toggle";
import { LuCheck, LuChevronDown, LuChevronUp } from "react-icons/lu";
import clsx from "clsx";

interface DropdownOption {
  value: string;
  label: string;
}

interface DropdownProps {
  options: DropdownOption[];
  defaultOption: string;
  selectedOption: string;
  selectedDirection: "asc" | "desc" | string;
  onDirectionChange: (direction: "asc" | "desc") => void;
  onValueChange: (option: string) => void;
}

const Dropdown = ({ ...props }: DropdownProps) => (
  <div className="inline-flex h-[35px] w-max items-center justify-center rounded-lg bg-slate-800 text-[13px] leading-none text-violet11 shadow-[0_2px_10px] shadow-black/10 outline-none hover:bg-mauve3 focus:shadow-black">
    <Select.Root value={props.selectedOption} {...props}>
      <Select.Trigger
        className="h-[35px] outline-none group inline-flex items-center justify-center gap-[5px] rounded-lg bg-slate-800 text-[13px] pl-[15px] pr-[15px] hover:bg-slate-900"
        aria-label="Dropdown Option"
      >
        <Select.Value placeholder="Loading" />
      </Select.Trigger>
      <Select.Portal>
        <Select.Content className="overflow-hidden rounded-lg bg-slate-800 shadow-[0px_10px_38px_-10px_rgba(22,_23,_24,_0.35),0px_10px_20px_-15px_rgba(22,_23,_24,_0.2)]">
          <Select.ScrollUpButton className="flex h-[25px] cursor-default items-center justify-center bg-slate-700">
            <LuCheck />
          </Select.ScrollUpButton>
          <Select.Viewport className="p-[5px]">
            <Select.Group>
              {props.options.map((option, i) => (
                <SelectItem
                  key={JSON.stringify(option) + i}
                  value={option.value}
                  className="data-[disabled]:pointer-events-none data-[highlighted]:bg-aoi-700 data-[disabled]:text-mauve8 data-[highlighted]:outline-none"
                >
                  {option.label}
                </SelectItem>
              ))}
            </Select.Group>
          </Select.Viewport>
          <Select.ScrollDownButton className="flex h-[25px] cursor-default items-center justify-center bg-white text-violet11">
            <LuChevronDown />
          </Select.ScrollDownButton>
        </Select.Content>
      </Select.Portal>
    </Select.Root>
    <div className="border-l border-neutral-500 h-2/3" />
    <Toggle.Root
      aria-label="Toggle sort direction"
      pressed={props.selectedDirection === "desc"}
      onPressedChange={(toggle) =>
        props.onDirectionChange(toggle ? "desc" : "asc")
      }
      className="flex size-[35px] items-center justify-center rounded-lg bg-slate-800 hover:bg-slate-900"
    >
      {props.selectedDirection === "asc" ? (
        <LuChevronUp className="transition-all duration-300 inline-flex h-[35px]" />
      ) : (
        <LuChevronDown className="transition-all duration-300 inline-flex h-[35px]" />
      )}
    </Toggle.Root>
  </div>
);

const SelectItem = React.forwardRef(
  (
    { children, className, direction, changeDirection, ...props }: any,
    forwardedRef,
  ) => {
    return (
      <Select.Item
        className={clsx(
          "relative flex h-[25px] select-none items-center rounded-md pl-[25px] pr-[35px] text-[13px] leading-none text-violet11 data-[disabled]:pointer-events-none data-[highlighted]:bg-aoi-700 data-[disabled]:text-mauve8 data-[highlighted]:outline-none",
          className,
        )}
        {...props}
        ref={forwardedRef}
      >
        <Select.ItemText>{children}</Select.ItemText>
        <Select.ItemIndicator className="absolute right-[10px] top-[50%] translate-y-[-50%]">
          <LuCheck />
        </Select.ItemIndicator>
      </Select.Item>
    );
  },
);

export default Dropdown;

"use client";

import { Group, Stepper, type StepperProps } from "@mantine/core";
import type { ReactNode } from "react";
import { AppButton } from "./button";

export type StepperPanelStep = {
  children: ReactNode;
  description?: string;
  label: string;
};

type StepperPanelProps = Omit<StepperProps, "children" | "active" | "onStepClick"> & {
  active: number;
  onBack?: () => void;
  onNext?: () => void;
  onStepClick?: (step: number) => void;
  steps: StepperPanelStep[];
};

export const StepperPanel = ({
  active,
  onBack,
  onNext,
  onStepClick,
  steps,
  ...props
}: StepperPanelProps) => (
  <>
    <Stepper active={active} onStepClick={onStepClick} {...props}>
      {steps.map((step) => (
        <Stepper.Step key={step.label} label={step.label} description={step.description}>
          {step.children}
        </Stepper.Step>
      ))}
    </Stepper>
    <Group position="right" mt="md">
      {onBack ? (
        <AppButton appVariant="secondary" onClick={onBack} disabled={active === 0}>
          Back
        </AppButton>
      ) : null}
      {onNext ? <AppButton onClick={onNext}>Next</AppButton> : null}
    </Group>
  </>
);

import { createFormHook, formOptions } from "@tanstack/react-form";

import { ComboboxField, SelectField, SubmitButton, TextareaField, TextField } from "./components";
import { fieldContext, formContext, useFieldContext, useFormContext } from "./form-context";

// Create the app-wide form hook with pre-bound components
// This provides type-safe form handling with reusable field components
const { useAppForm, withForm } = createFormHook({
  fieldContext,
  formContext,
  fieldComponents: {
    ComboboxField,
    TextField,
    TextareaField,
    SelectField,
  },
  formComponents: {
    SubmitButton,
  },
});

// Re-export everything for convenient imports
export {
  // Pre-built field components
  ComboboxField,
  formOptions,
  SelectField,
  SubmitButton,
  TextareaField,
  TextField,
  // Form hook and utilities
  useAppForm,
  // Context hooks for custom components
  useFieldContext,
  useFormContext,
  withForm,
};

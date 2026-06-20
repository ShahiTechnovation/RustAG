"use client";

import { useState, type FormEvent } from "react";
import { motion } from "motion/react";
import { ArrowUpRight, CheckCircle2, Loader2 } from "lucide-react";

import { Button, Eyebrow, Field, GridBackground, Input, RingField } from "@/components/ui";

const ease = [0.22, 1, 0.36, 1] as const;
const FOLLOW_URL = "https://x.com/RustAG_xyz";

type FieldErrors = Partial<Record<"email" | "wallet" | "twitter" | "followed", string>>;

export default function EarlyAccessPage() {
  const [submitting, setSubmitting] = useState(false);
  const [done, setDone] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [errors, setErrors] = useState<FieldErrors>({});
  const [followed, setFollowed] = useState(false);

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setFormError(null);
    setErrors({});

    const form = new FormData(event.currentTarget);
    const payload = {
      email: String(form.get("email") ?? ""),
      wallet: String(form.get("wallet") ?? ""),
      twitter: String(form.get("twitter") ?? ""),
      followed,
    };

    setSubmitting(true);
    try {
      const res = await fetch("/api/early-access", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(payload),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        setErrors((data?.errors as FieldErrors) ?? {});
        setFormError(data?.error ?? "Something went wrong. Please try again.");
        return;
      }
      setDone(true);
    } catch {
      setFormError("Network error. Please try again.");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <section className="relative overflow-hidden px-6 pb-28 pt-36 sm:pt-40">
      <RingField />
      <GridBackground className="opacity-40" />

      <div className="mx-auto max-w-xl">
        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, ease }}
          className="text-center"
        >
          <Eyebrow className="justify-center text-brand">Private Beta · Early Access</Eyebrow>
          <h1 className="font-display mt-6 text-balance text-4xl font-bold leading-[1.02] tracking-tight text-fg sm:text-5xl">
            Get on the <em className="font-serif italic font-normal text-brand">early access</em>{" "}
            list.
          </h1>
          <p className="mx-auto mt-5 max-w-md text-pretty text-base leading-relaxed text-muted">
            RustAG is in private beta and the dashboard isn&apos;t open to everyone yet. Drop your
            details and we&apos;ll reach out as we roll out access.
          </p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease, delay: 0.12 }}
          className="mt-10 rounded-card border border-border-strong bg-surface/60 p-6 backdrop-blur-sm sm:p-8"
        >
          {done ? (
            <div className="flex flex-col items-center gap-4 py-8 text-center">
              <CheckCircle2 size={44} className="text-brand" />
              <h2 className="font-display text-2xl font-bold tracking-tight text-fg">
                You&apos;re on the list.
              </h2>
              <p className="max-w-sm text-sm leading-relaxed text-muted">
                Thanks for your interest in RustAG. We&apos;ll email you at the address you provided
                as we open up access. Keep an eye on{" "}
                <a
                  href={FOLLOW_URL}
                  target="_blank"
                  rel="noreferrer"
                  className="text-brand hover:text-brand-strong"
                >
                  @RustAG_xyz
                </a>{" "}
                for updates.
              </p>
            </div>
          ) : (
            <form onSubmit={onSubmit} className="space-y-5" noValidate>
              <Field label="Email" hint={errors.email}>
                <Input
                  name="email"
                  type="email"
                  autoComplete="email"
                  placeholder="you@domain.com"
                  required
                  aria-invalid={!!errors.email}
                />
              </Field>

              <Field label="Solana wallet address" hint={errors.wallet}>
                <Input
                  name="wallet"
                  placeholder="9xQe…F4kP"
                  required
                  spellCheck={false}
                  className="font-mono"
                  aria-invalid={!!errors.wallet}
                />
              </Field>

              <Field label="X (Twitter) handle" hint={errors.twitter}>
                <Input
                  name="twitter"
                  placeholder="@yourhandle"
                  required
                  spellCheck={false}
                  aria-invalid={!!errors.twitter}
                />
              </Field>

              <div className="rounded-[3px] border border-border bg-white/[0.02] p-4">
                <label className="flex items-start gap-3">
                  <input
                    type="checkbox"
                    checked={followed}
                    onChange={(e) => setFollowed(e.target.checked)}
                    className="mt-0.5 size-4 shrink-0 accent-brand"
                    aria-invalid={!!errors.followed}
                  />
                  <span className="text-sm leading-relaxed text-muted">
                    I follow{" "}
                    <a
                      href={FOLLOW_URL}
                      target="_blank"
                      rel="noreferrer"
                      className="inline-flex items-center gap-0.5 font-medium text-brand hover:text-brand-strong"
                    >
                      @RustAG_xyz
                      <ArrowUpRight size={13} />
                    </a>{" "}
                    on X for launch updates.
                  </span>
                </label>
                {errors.followed ? (
                  <p className="mt-2 pl-7 text-xs text-state-dirty">{errors.followed}</p>
                ) : null}
              </div>

              {formError ? (
                <p className="text-sm text-red-400" role="alert">
                  {formError}
                </p>
              ) : null}

              <Button type="submit" size="lg" className="w-full" disabled={submitting}>
                {submitting ? (
                  <>
                    <Loader2 size={16} className="animate-spin" />
                    Submitting
                  </>
                ) : (
                  "Request early access"
                )}
              </Button>

              <p className="text-center text-xs text-faint">
                We&apos;ll only use these details to contact you about RustAG access.
              </p>
            </form>
          )}
        </motion.div>
      </div>
    </section>
  );
}

import { OraclePrices } from "@/components/OraclePrices";
import { CTASection } from "@/components/marketing/CTASection";
import { FeatureBento } from "@/components/marketing/FeatureBento";
import { Hero } from "@/components/marketing/Hero";
import { HowItWorks } from "@/components/marketing/HowItWorks";
import { LogoTicker } from "@/components/marketing/LogoTicker";
import { MirrorExplainer } from "@/components/marketing/MirrorExplainer";
import { StatsBand } from "@/components/marketing/StatsBand";
import { TerminalShowcase } from "@/components/marketing/TerminalShowcase";

export default function LandingPage() {
  return (
    <>
      <Hero />
      <section className="px-6 pb-6">
        <OraclePrices compact />
      </section>
      <LogoTicker />
      <MirrorExplainer />
      <FeatureBento />
      <TerminalShowcase />
      <HowItWorks />
      <StatsBand />
      <CTASection />
    </>
  );
}

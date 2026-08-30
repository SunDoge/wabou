---
name: frontend-design-review
description: >
  Review and create distinctive, production-grade frontend interfaces with high design quality and design system compliance.
  Evaluates using three pillars: frictionless insight-to-action, quality craft, and trustworthy building.
  USE FOR: PR reviews, design reviews, accessibility audits, design system compliance checks, creative frontend design,
  UI code review, component reviews, responsive design checks, theme testing, and creating memorable UI.
  DO NOT USE FOR: Backend API reviews, database schema reviews, infrastructure or DevOps work, pure business logic
  without UI, or non-frontend code.
---

# Frontend Design Review

Review UI implementations against design quality standards and your design system **OR** create distinctive, production-grade frontend interfaces from scratch.

## Two Modes

### Mode 1: Design Review
Evaluate existing UI for design system compliance, three quality pillars (Frictionless, Quality Craft, Trustworthy), accessibility, and code quality.

### Mode 2: Creative Frontend Design
Create distinctive interfaces that avoid generic "AI slop" aesthetics, have clear conceptual direction, and execute with precision.

---

## Creative Frontend Design

Before coding, establish a compact design read:
- **User and task**: Who uses this surface, and what must they understand or finish?
- **Product mode**: workspace, utility, content, commerce, editorial, or immersive experience.
- **Information hierarchy**: What must be visible first, what is secondary, and what can be disclosed later?
- **Tone**: Choose a precise visual direction that fits the product rather than a fashionable default.
- **Constraints**: Framework, platforms, input modes, minimum viewport, performance, and accessibility.
- **Differentiation**: Name one product-specific visual or interaction idea worth remembering.

### Aesthetics Guidelines

- **Typography**: Use an intentional type hierarchy and readable measures. A distinctive font is useful only when it improves product fit and platform rendering remains reliable.
- **Color & Theme**: Use semantic roles, sufficient contrast, and a restrained accent hierarchy. Treat light and dark themes as separate rendered states.
- **Motion**: Explain state or spatial continuity. Do not delay interaction, repeatedly animate stable content, or use motion to conceal remounting.
- **Spatial Composition**: Choose deliberate density, alignment, containment, and whitespace. Cards, asymmetry, overlap, or decorative depth require a semantic purpose.
- **Surfaces**: Use borders, elevation, translucency, texture, and gradients only when they clarify grouping, focus, or hierarchy.

**AVOID**: Cookie-cutter dashboards, card-per-section layouts, decorative gradients, arbitrary radii, competing primary actions, fake data presented as product state, and internal implementation language in user-facing copy.

Match implementation complexity to vision. Maximalist = elaborate code. Minimalist = restraint and precision.

---

## Design Review

### Repository profiles

When this skill is used in the Wabou repository, read and apply
[references/wabou-review.md](references/wabou-review.md). That profile replaces
browser-, Figma-, and Storybook-specific proof with Wabou's component, layout,
semantic, native-behavior, and pixel evidence.

### Design System Workflow

**Before implementing:**
1. Review component in your Storybook / component library for API and usage
2. Use Figma Dev Mode to get exact specs (spacing, tokens, properties)
3. Implement using design system components + design tokens

**During review:**
1. Compare implementation to Figma design
2. Verify design tokens are used (not hardcoded values)
3. Check all variants/states are implemented correctly
4. Flag deviations (needs design approval)

**If component doesn't exist:**
1. Check if existing component can be adapted
2. Reach out to design for new component creation
3. Document exception and rationale in code

### Review Process

1. Identify user task
2. Check design system for matching patterns
3. Evaluate aesthetic direction
4. Identify scope (component, feature, or flow)
5. Enumerate relevant initial, loading, empty, success, error, disabled, and recovery states
6. Evaluate each pillar
7. Pressure-test long content, constrained space, alternate theme, and relevant input methods
8. Score and prioritize issues (blocking/major/minor)
9. Provide recommendations with design system examples and observable proof

### Core Principles

- **Task completion**: Minimum clicks. Every screen answers "What can I do?" and "What happens next?"
- **Action hierarchy**: 1-2 primary actions per view. Progressive disclosure for secondary.
- **Onboarding**: Explain features on introduction. Smart defaults over configuration.
- **Navigation**: Clear entry/exit points. Back/cancel always available. Breadcrumbs for deep flows.

---

## Quality Pillars

### 1. Frictionless Insight to Action

**Evaluate:** Task completable in ≤3 interactions? Primary action obvious and singular?

**Red flags:** Excessive clicks, multiple competing primary buttons, buried actions, dead ends.

### 2. Quality is Craft

**Evaluate:**
- Design system compliance: matches Figma specs, uses design tokens
- Aesthetic direction: distinctive typography, cohesive colors, intentional motion
- Accessibility: WCAG 2.2 AA where the platform exposes the required semantics; document native-framework gaps instead of implying compliance

**Red flags:** Generic AI aesthetics, hardcoded values, implementation doesn't match Figma, broken reflow, missing focus indicators.

### 3. Trustworthy Building

**Evaluate:**
- AI transparency: disclaimer on AI-generated content
- Error transparency: actionable error messages

**Red flags:** Missing AI disclaimers, opaque errors without guidance.

---

## Review Output Format

See [references/review-output-format.md](references/review-output-format.md) for the full review template.

## Review Type Modifiers

See [references/review-type-modifiers.md](references/review-type-modifiers.md) for context-specific review focus areas (PR, Creative, Design, Accessibility).

## Quick Checklist

See [references/quick-checklist.md](references/quick-checklist.md) for the pre-approval checklist covering design system compliance, aesthetic quality, frictionless, quality craft, and trustworthy pillars.

## Pattern Examples

See [references/pattern-examples.md](references/pattern-examples.md) for good/bad examples of creative frontend and design system review work.

---

## Acknowledgments

Creative frontend principles inspired by [Anthropic's frontend-design skill](https://github.com/anthropics/skills/tree/main/skills/frontend-design). Design review principles and quality pillar framework created by [@Quirinevwm](https://github.com/Quirinevwm) for systematic UI evaluation.
Context-fit design, complete-state review, and rendered-verification principles adapted from [PracticalSwan/agent-skills](https://github.com/PracticalSwan/agent-skills/tree/main/frontend-design), licensed MIT AND Apache-2.0.

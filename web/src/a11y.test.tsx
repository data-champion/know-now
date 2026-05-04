import { render } from "@testing-library/react";
import { axe } from "vitest-axe";
import { describe, expect, test } from "vitest";
import { SearchInput } from "./components/SearchInput";
import { EntityDetail } from "./components/EntityDetail";
import { RelationshipTable } from "./components/RelationshipTable";

describe("WCAG 2.2 AA — axe-core audits", () => {
  test("SearchInput has no violations", async () => {
    const { container } = render(
      <SearchInput value="" onChange={() => {}} placeholder="Search..." />,
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  test("EntityDetail has no violations", async () => {
    const entity = {
      id: "ent_test",
      name: "Test Entity",
      domain: "core",
      description: "A test entity for accessibility",
      attributes: [
        { id: "attr_1", name: "id", logical_type: "integer", description: "Primary key" },
        { id: "attr_2", name: "email", logical_type: "string", description: "Email" },
      ],
    };
    const { container } = render(
      <EntityDetail entity={entity} onClose={() => {}} />,
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  test("RelationshipTable has no violations", async () => {
    const relationships = [
      { id: "rel_1", from_entity: "user", to_entity: "order", cardinality: "1:N", description: "User places orders" },
      { id: "rel_2", from_entity: "order", to_entity: "product", cardinality: "N:M", description: null },
    ];
    const { container } = render(
      <RelationshipTable relationships={relationships} />,
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  test("RelationshipTable empty state has no violations", async () => {
    const { container } = render(
      <RelationshipTable relationships={[]} />,
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});

describe("WCAG 2.2 AA — keyboard navigation", () => {
  test("SearchInput is focusable", () => {
    const { container } = render(
      <SearchInput value="" onChange={() => {}} placeholder="Search entities..." />,
    );
    const input = container.querySelector("input");
    expect(input).not.toBeNull();
    expect(input?.getAttribute("role")).toBe("searchbox");
    expect(input?.getAttribute("aria-label")).toBe("Search entities...");
  });

  test("EntityDetail close button has aria-label", () => {
    const entity = {
      id: "ent_1",
      name: "User",
      domain: null,
      description: null,
      attributes: [],
    };
    const { container } = render(
      <EntityDetail entity={entity} onClose={() => {}} />,
    );
    const button = container.querySelector("button");
    expect(button?.getAttribute("aria-label")).toBe("Close detail view");
  });

  test("RelationshipTable uses proper scope attributes", () => {
    const relationships = [
      { id: "r1", from_entity: "a", to_entity: "b", cardinality: null, description: null },
    ];
    const { container } = render(
      <RelationshipTable relationships={relationships} />,
    );
    const ths = container.querySelectorAll("th");
    for (const th of ths) {
      expect(th.getAttribute("scope")).toBe("col");
    }
  });
});

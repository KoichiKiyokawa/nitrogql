import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { Hint } from "@/app/_utils/Hint";
import Link from "next/link";
import { ogp } from "@/app/_utils/metadata";

export const metadata = ogp({
  title: "Monorepo Guide",
});

export default function GettingStarted() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "Guides", href: "/guides" }]}
          current="Monorepo Guide"
        />
        <h2>Monorepo Guide</h2>

        <p>
          This page guides you how to configure nitrogql for monorepo projects.
          In a monorepo setting, you may have GraphQL schema and operations in
          different packages. This guide shows you how to use nitrogql in such
          projects.
        </p>

        <p>
          Basically you set up nitrogql in each package that contains GraphQL
          files. Appropriate configuration for schema packages and operation
          packages are different.
        </p>

        <h3 id="schema-package">Schema Package</h3>
        <p>
          Here, we assume that you have a package that contains your GraphQL
          schema.
        </p>
        <p>
          In such package, you only generate code for the schema. Your
          configuration file should look like:
        </p>
        <Highlight language="yaml">
          {`schema: ./schema/*.graphql
extensions:
  nitrogql:
    generate:
      schemaOutput: ./dist/schema.d.ts
`}
        </Highlight>
        <Hint>
          💡 See{" "}
          <Link href="/configuration/options">Configuration Options</Link> for
          all available options.
        </Hint>

        <p>
          By running <code>nitrogql generate</code>, you will get a{" "}
          <code>schema.d.ts</code> file in the <code>dist</code> directory.
        </p>

        <p>
          The schema package should export the generated schema types so that
          other packages can use them. You can do this by exporting the
          generated schema types from the the package.
        </p>
        <Highlight language="typescript">
          {`// packages/schema-package/index.ts
export * from "./dist/schema";
`}
        </Highlight>

        <h3 id="operation-packages">Operation Packages</h3>
        <p>
          In the operation packages, you generate code for operations. Your
          configuration file should look like:
        </p>
        <Highlight language="yaml">
          {`schema: ../schema-package/schema/*.graphql
documents: ./app/**/*.graphql
extensions:
  nitrogql:
    generate:
      schemaModuleSpecifier: "@your-org/schema-package"
`}
        </Highlight>
        <p>
          Note that the <code>schema</code> option is still required, and should
          point to <code>.graphql</code> files that contain the schema
          definitions. You need to use relative paths that go beyond the package
          boundary.
        </p>
        <p>
          You must also specify the <code>schemaModuleSpecifier</code> option.
          This option tells nitrogql to import the generated schema types using
          the specified module specifier. This option is required if you want to
          only generate code for operations.
        </p>
        <p>
          By running <code>nitrogql generate</code>, you will get TypeScript
          files generated next to your GraphQL files.
        </p>
        <p>
          As the <code>schemaModuleSpecifier</code> option suggests, operation
          packages will depend on the schema package.
        </p>

        <h3 id="using-the-generated-code">GraphQL Server Packages</h3>
        <p>
          Currently, nitrogql does not support generating code for implementing
          GraphQL servers.
        </p>
        <p>
          Support for GraphQL servers is on the roadmap with the highest
          priority.
        </p>
      </main>
    </Toc>
  );
}
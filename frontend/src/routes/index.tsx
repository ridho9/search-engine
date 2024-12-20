import * as React from "react";
import {
  createFileRoute,
  useNavigate,
  useSearch,
} from "@tanstack/react-router";
import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { InfoIcon } from "lucide-react";

type HomeSearch = {
  query?: string;
};

export const Route = createFileRoute("/")({
  component: HomeComponent,
  validateSearch: (search: Record<string, unknown>): HomeSearch => {
    return {
      query: search["query"] as string,
    };
  },
});

type EngineQueryResponse = {
  q: string;
  elapsed_ms: number;
  hits: {
    score: number;
    doc: { url: string; title: string; body: string[] };
    relevant_body: string[];
  }[];
  count?: number;
};

function HomeComponent() {
  const navigate = useNavigate({ from: Route.fullPath });
  const queryClient = useQueryClient();
  const search = Route.useSearch();
  const [queryTerm, setQuery] = useState<string | undefined>(search.query);

  const q = useQuery({
    queryKey: ["search-doc", search.query],
    queryFn: async () => {
      if (!search.query) return;

      const startTime = performance.now();

      const baseUrl =
        (import.meta as any).env.VITE_ENGINE_HOST || "http://localhost:6969";
      const param = new URLSearchParams();
      param.set("query", search.query);
      const url = `${baseUrl}/api/docs?${param.toString()}`;

      const resp = await fetch(url);
      const endTime = performance.now();
      const reqTime = endTime - startTime;

      if (resp.status !== 200) throw Error(`query error: ${await resp.text()}`);

      const json = (await resp.json()) as EngineQueryResponse;

      return { ...json, client_ms: reqTime };
    },
    refetchOnWindowFocus: false,
    retry: false,
  });

  const submitForm = (ev: React.FormEvent<HTMLFormElement>) => {
    ev.preventDefault();
    queryClient.invalidateQueries({ queryKey: ["search-doc", search.query] });
    navigate({
      search: (prev) => ({ ...prev, query: queryTerm }),
    });
  };

  return (
    <div className="">
      <h1 className="font-bold text-3xl mb-1">Search Docs</h1>
      <form
        className="my-4 flex flex-col max-w-lg space-y-2"
        onSubmit={submitForm}
      >
        <input
          name="query"
          type="text"
          id="query"
          className="input input-bordered"
          value={queryTerm}
          onChange={(e) => setQuery(e.target.value)}
        ></input>
        <button className="btn w-48" type="submit">
          Search
        </button>
      </form>

      <hr />

      <div className="my-2">
        {search.query && q.isFetching && <p>Loading...</p>}
        {search.query && q.isError && <p>Error: {q.error.message}</p>}
        {q.isSuccess && <ResultBody q={q.data} />}
      </div>
    </div>
  );
}

function ResultBody(params: {
  q?: EngineQueryResponse & { client_ms: number };
}) {
  const { q } = params;

  if (!q) return <></>;

  return (
    <>
      {q.count !== undefined && <p>Documents matches: {q.count}</p>}
      <p className="flex items-center">
        Internal time: {q?.elapsed_ms.toFixed(3)}ms{" "}
        <span title="time taken by the engine to query" className="ml-2">
          <InfoIcon size={16} />
        </span>
      </p>
      <p className="flex items-center">
        Client time: {q?.client_ms.toFixed(3)}ms
        <span title="end-to-end time taken" className="ml-2">
          <InfoIcon size={16} />
        </span>
      </p>

      {q.hits.map((hit) => {
        const show_body = hit.relevant_body
          .filter((s) => s.trim() != "")
          .join(" ")
          .slice(0, 300);

        return (
          <div
            key={hit.doc.url[0]}
            className="mt-4 card card-bordered bg-gray-800 max-w-5xl"
          >
            <a href={hit.doc.url} className="card-body">
              <p className="card-title font-bold text-lg">{hit.doc.title}</p>
              <p className="text-sm">{hit.doc.url}</p>
              <p className="">{show_body}...</p>
            </a>
          </div>
        );
      })}
    </>
  );
}

/**
 * Ambient type declarations for third-party browser globals.
 *
 * These libraries are loaded via <script> tags, not ES module imports,
 * so we declare just enough surface area for tsc --checkJs to pass.
 */

/** MapLibre GL JS global — loaded from CDN <script> tag. */
declare const maplibregl: {
  Map: new (options: Record<string, unknown>) => maplibregl.Map;
  NavigationControl: new (options?: Record<string, unknown>) => unknown;
  Popup: new (options?: Record<string, unknown>) => maplibregl.Popup;
  LngLatBounds: new (...args: unknown[]) => maplibregl.LngLatBounds;
  addProtocol(name: string, handler: unknown): void;
};

declare namespace maplibregl {
  interface Map {
    on(event: string, handler: Function): Map;
    on(event: string, layer: string, handler: Function): Map;
    addControl(control: unknown, position?: string): Map;
    removeControl(control: unknown): Map;
    addSource(id: string, source: Record<string, unknown>): Map;
    getSource(id: string): unknown;
    addLayer(layer: Record<string, unknown>): Map;
    setLayoutProperty(layer: string, name: string, value: unknown): Map;
    setPaintProperty(layer: string, name: string, value: unknown): Map;
    getCanvas(): HTMLCanvasElement;
    fitBounds(bounds: LngLatBounds | [[number, number], [number, number]], options?: Record<string, unknown>): Map;
    setProjection(projection: Record<string, unknown>): Map;
  }
  interface Popup {
    setLngLat(lnglat: [number, number]): Popup;
    setHTML(html: string): Popup;
    addTo(map: Map): Popup;
    remove(): Popup;
  }
  interface LngLatBounds {
    extend(lnglat: [number, number]): LngLatBounds;
    isEmpty(): boolean;
  }
}

/** PMTiles protocol — loaded from CDN <script> tag. */
declare const pmtiles: {
  Protocol: new () => { tile: unknown };
};

/** Protomaps basemaps theme helper — loaded from CDN <script> tag. */
declare const basemaps: {
  layers(source: string, flavor: unknown, options?: Record<string, unknown>): unknown[];
  namedFlavor(name: string): unknown;
};

/** TopoJSON global — loaded from CDN <script> tag. */
declare const topojson: {
  feature(topology: unknown, object: unknown): GeoJSON.FeatureCollection;
};

/** GeoJSON namespace for feature/geometry types used in stats-map.js. */
declare namespace GeoJSON {
  interface Feature {
    id?: string | number;
    type: string;
    geometry: Geometry;
    properties: Record<string, unknown> | null;
  }
  interface Geometry {
    type: string;
    coordinates: unknown;
  }
  interface FeatureCollection {
    type: string;
    features: Feature[];
  }
}

/**
 * Ambient type declarations for third-party browser globals.
 *
 * These libraries are loaded via <script> tags, not ES module imports,
 * so we declare just enough surface area for tsc --checkJs to pass.
 */

/** Leaflet global — loaded from CDN <script> tag. */
declare const L: {
  map(id: string, options?: Record<string, unknown>): L.Map;
  tileLayer(url: string, options?: Record<string, unknown>): L.TileLayer;
  marker(latlng: [number, number]): L.Marker;
  polyline(latlngs: [number, number][], options?: Record<string, unknown>): L.Polyline;
  circleMarker(latlng: [number, number], options?: Record<string, unknown>): L.CircleMarker;
  layerGroup(): L.LayerGroup;
  geoJSON(data: unknown, options?: Record<string, unknown>): L.GeoJSON;
  control(options?: Record<string, unknown>): L.Control;
  DomUtil: {
    create(tagName: string, className?: string): HTMLElement;
  };
  Browser: {
    ie: boolean;
    opera: boolean;
    edge: boolean;
  };
};

declare namespace L {
  interface Map {
    setView(center: [number, number], zoom: number): Map;
    fitBounds(bounds: [number, number][], options?: Record<string, unknown>): Map;
    addLayer(layer: unknown): Map;
    removeLayer(layer: unknown): Map;
  }
  interface TileLayer {
    addTo(map: Map): TileLayer;
  }
  interface Marker {
    addTo(map: Map): Marker;
  }
  interface Polyline {
    addTo(group: LayerGroup | Map): Polyline;
    bindPopup(content: string, options?: Record<string, unknown>): Polyline;
  }
  interface CircleMarker {
    addTo(group: LayerGroup | Map): CircleMarker;
    bindPopup(content: string, options?: Record<string, unknown>): CircleMarker;
    on(event: string, handler: Function): CircleMarker;
    openPopup(): CircleMarker;
    closePopup(): CircleMarker;
    _popupHandlingClick?: boolean;
  }
  interface LayerGroup {
    addTo(map: Map): LayerGroup;
    clearLayers(): LayerGroup;
  }
  interface GeoJSON {
    addTo(map: Map): GeoJSON;
  }
  interface Control {
    addTo(map: Map): Control;
    onAdd?: (map: Map) => HTMLElement;
    update?: (...args: unknown[]) => void;
    _div?: HTMLElement;
  }
}

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

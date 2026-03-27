/**
 * Shared type declarations for Travel Mapper static JS files.
 *
 * These are ambient declarations consumed via `/// <reference path="types.d.ts" />`
 * in the JS modules — they are NOT served to browsers.
 */

/** A single travel hop as returned by the `/journeys` API. */
interface HopResponse {
  /** Database identifier. */
  id: number;
  /** Mode of transport: "air" | "rail" | "boat" | "transport". */
  travel_type: string;
  /** Name of the origin location. */
  origin_name: string;
  /** Latitude of the origin. */
  origin_lat: number;
  /** Longitude of the origin. */
  origin_lng: number;
  /** Name of the destination location. */
  dest_name: string;
  /** Latitude of the destination. */
  dest_lat: number;
  /** Longitude of the destination. */
  dest_lng: number;
  /** Departure date (YYYY-MM-DD). */
  start_date: string;
  /** Arrival date (YYYY-MM-DD). */
  end_date: string;
  /** Carrier name or IATA code (e.g. "BA", "Amtrak"). */
  carrier?: string;
  /** Live flight status (e.g. "scheduled", "active", "landed", "cancelled"). */
  status?: string;
  /** Delay in minutes (positive = late, negative = early). */
  delay_minutes?: number;
  /** Departure gate from status enrichment. */
  dep_gate?: string;
  /** Departure terminal from status enrichment. */
  dep_terminal?: string;
  /** Arrival gate from status enrichment. */
  arr_gate?: string;
  /** Arrival terminal from status enrichment. */
  arr_terminal?: string;
  /** Departure platform from rail status enrichment. */
  dep_platform?: string;
  /** Arrival platform from rail status enrichment. */
  arr_platform?: string;
  /** Whether the route was verified via ADS-B data from OpenSky Network. */
  route_verified?: boolean;
}

/** An aggregated route between two cities, used for map rendering. */
interface AggregatedRoute {
  /** [lat, lng] of the origin. */
  from: [number, number];
  /** [lat, lng] of the destination. */
  to: [number, number];
  /** Display name of the origin city. */
  origin_name: string;
  /** Display name of the destination city. */
  dest_name: string;
  /** Individual hops along this route. */
  hops: HopResponse[];
}

/** An aggregated city node on the map with visit counts and connections. */
interface CityNode {
  /** Display name of the city. */
  name: string;
  /** Latitude. */
  lat: number;
  /** Longitude. */
  lng: number;
  /** Total appearances (as origin or destination). */
  count: number;
  /** Destination name -> frequency. */
  routes: Record<string, number>;
}

/** Country visit counts keyed by ISO 3166-1 alpha-2 code. */
type CountryCounts = Record<string, number>;

/** Active filter parameters for the journey search. */
interface FilterParams {
  /** Free-text search query. */
  q?: string;
  /** Travel type filter. */
  type?: string;
  /** Origin name filter. */
  origin?: string;
  /** Destination name filter. */
  dest?: string;
  /** Start date lower bound (YYYY-MM-DD). */
  date_from?: string;
  /** Start date upper bound (YYYY-MM-DD). */
  date_to?: string;
  /** Airline filter. */
  airline?: string;
  /** Cabin class filter. */
  cabin_class?: string;
  /** Flight reason filter. */
  flight_reason?: string;
}

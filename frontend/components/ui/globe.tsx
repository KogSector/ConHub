"use client";

import { cn } from "@/lib/utils";
import createGlobe from "cobe";
import { FC, HTMLAttributes, useEffect, useRef } from "react";

interface GlobeProps extends HTMLAttributes<HTMLCanvasElement> {
	width?: number;
	height?: number;
}

export const Globe: FC<GlobeProps> = ({
	className,
	width = 600,
	height = 600,
	...props
}) => {
	const canvasRef = useRef<HTMLCanvasElement>(null);

	useEffect(() => {
		let phi = 0;
		const globe = createGlobe(canvasRef.current!, {
			devicePixelRatio: 2,
			width: width * 2,
			height: height * 2,
			phi: 0,
			theta: 0,
			dark: 1,
			diffuse: 1.2,
			mapSamples: 16000,
			mapBrightness: 6,
			baseColor: [0.3, 0.3, 0.3],
			markerColor: [0.1, 0.8, 1],
			glowColor: [1, 1, 1],
			markers: [
				{ location: [37.7595, -122.4367], size: 0.03 },
				{ location: [40.7128, -74.006], size: 0.1 },
			],
			onRender: (state) => {
				state.phi = phi;
				phi += 0.01;
			},
		});

		return () => {
			globe.destroy();
		};
	}, []);

	return (
		<canvas
			ref={canvasRef}
			style={{
				width: width,
				height: height,
				maxWidth: "100%",
				aspectRatio: "1/1",
			}}
			className={cn(
				"h-auto w-full max-w-[600px]",
				className,
			)}
			{...props}
		/>
	);
};

#pragma version(1)
#pragma rs java_package_name(net.sourceforge.opencamera)
#pragma rs_fp_relaxed

rs_allocation bitmap0;
rs_allocation bitmap2;

int offset_x0 = 0, offset_y0 = 0;
int offset_x2 = 0, offset_y2 = 0;

float parameter_A0 = 1.0f;
float parameter_B0 = 0.0f;
float parameter_A1 = 1.0f;
float parameter_B1 = 0.0f;
float parameter_A2 = 1.0f;
float parameter_B2 = 0.0f;

const float weight_scale_c = (float)((1.0-1.0/127.5)/127.5);

const int tonemap_algorithm_clamp_c = 0;
const int tonemap_algorithm_exponential_c = 1;
const int tonemap_algorithm_reinhard_c = 2;
const int tonemap_algorithm_filmic_c = 3;
const int tonemap_algorithm_aces_c = 4;

int tonemap_algorithm = tonemap_algorithm_reinhard_c;

// for Exponential:
const float exposure = 1.2f;

// for Reinhard:
float tonemap_scale = 1.0f;

// for Filmic Uncharted 2:
const float filmic_exposure_bias = 2.0f / 255.0f;
float W = 11.2f;

// for various:
float linear_scale = 1.0f;

static float Uncharted2Tonemap(float x) {
	const float A = 0.15f;
	const float B = 0.50f;
	const float C = 0.10f;
	const float D = 0.20f;
	const float E = 0.02f;
	const float F = 0.30f;
	return ((x*(A*x+C*B)+D*E)/(x*(A*x+B)+D*F))-E/F;
}

static uchar4 tonemap(float3 hdr) {
	// tonemap
	uchar4 out;
    switch( tonemap_algorithm )
	{
	    case tonemap_algorithm_clamp_c:
	    {
            // Simple clamp
            int r = (int)(hdr.r+0.5f);
            int g = (int)(hdr.g+0.5f);
            int b = (int)(hdr.b+0.5f);
            r = min(r, 255);
            g = min(g, 255);
            b = min(b, 255);
            out.r = r;
            out.g = g;
            out.b = b;
            out.a = 255;
            break;
        }
	    case tonemap_algorithm_exponential_c:
	    {
        	float3 out_f = linear_scale * 255.0f * (1.0 - exp( - exposure * hdr / 255.0f ));
            out.r = (uchar)clamp(out_f.r+0.5f, 0.0f, 255.0f);
            out.g = (uchar)clamp(out_f.g+0.5f, 0.0f, 255.0f);
            out.b = (uchar)clamp(out_f.b+0.5f, 0.0f, 255.0f);
            out.a = 255;
            break;
	    }
	    case tonemap_algorithm_reinhard_c:
	    {
            float value = fmax(hdr.r, hdr.g);
            value = fmax(value, hdr.b);
            float scale = 255.0f / ( tonemap_scale + value );
            scale *= linear_scale;
            // shouldn't need to clamp - linear_scale should be such that values don't map to more than 255
            out.r = (uchar)(scale * hdr.r + 0.5f);
            out.g = (uchar)(scale * hdr.g + 0.5f);
            out.b = (uchar)(scale * hdr.b + 0.5f);
            out.a = 255;
            /*int test_r = (int)(scale * hdr.r + 0.5f);
            int test_g = (int)(scale * hdr.g + 0.5f);
            int test_b = (int)(scale * hdr.b + 0.5f);
            if( test_r > 255 || test_g > 255 || test_b > 255 ) {
                out.r = 255;
                out.g = 0;
                out.b = 255;
            }*/
            break;
        }
	    case tonemap_algorithm_filmic_c:
	    {
            // Filmic Uncharted 2
            float white_scale = 255.0f / Uncharted2Tonemap(W);
            float curr_r = Uncharted2Tonemap(filmic_exposure_bias * hdr.r);
            float curr_g = Uncharted2Tonemap(filmic_exposure_bias * hdr.g);
            float curr_b = Uncharted2Tonemap(filmic_exposure_bias * hdr.b);
            curr_r *= white_scale;
            curr_g *= white_scale;
            curr_b *= white_scale;
            out.r = (uchar)clamp(curr_r+0.5f, 0.0f, 255.0f);
            out.g = (uchar)clamp(curr_g+0.5f, 0.0f, 255.0f);
            out.b = (uchar)clamp(curr_b+0.5f, 0.0f, 255.0f);
            out.a = 255;
            break;
        }
	    case tonemap_algorithm_aces_c:
	    {
	        const float a = 2.51f;
	        const float b = 0.03f;
	        const float c = 2.43f;
	        const float d = 0.59f;
	        const float e = 0.14f;
	        float3 x = hdr/255.0;
	        float3 out_f = 255.0f * (x*(a*x+b))/(x*(c*x+d)+e);
	        out.r = (uchar)clamp(out_f.r+0.5f, 0.0f, 255.0f);
	        out.g = (uchar)clamp(out_f.g+0.5f, 0.0f, 255.0f);
            out.b = (uchar)clamp(out_f.b+0.5f, 0.0f, 255.0f);
            out.a = 255;
            break;
	    }
	}

    /*
    // test
	if( x+offset_x0 < 0 || y+offset_y0 < 0 || x+offset_x0 >= rsAllocationGetDimX(bitmap0) || y+offset_y0 >= rsAllocationGetDimY(bitmap0) ) {
    	out.r = 255;
    	out.g = 0;
    	out.b = 255;
    	out.a = 255;
	}
	else if( x+offset_x2 < 0 || y+offset_y2 < 0 || x+offset_x2 >= rsAllocationGetDimX(bitmap2) || y+offset_y2 >= rsAllocationGetDimY(bitmap2) ) {
    	out.r = 255;
    	out.g = 255;
    	out.b = 0;
    	out.a = 255;
	}
	*/
    return out;
}

uchar4 __attribute__((kernel)) hdr(uchar4 in, uint32_t x, uint32_t y) {
    int32_t ix = x;
    int32_t iy = y;
    const int max_bitmaps_c = 3;
    int n_bitmaps = 3;
	const int mid_indx = (n_bitmaps-1)/2;
	uchar4 pixels[max_bitmaps_c];

	float parameter_A[max_bitmaps_c];
	float parameter_B[max_bitmaps_c];

    parameter_A[0] = parameter_A0;
    parameter_B[0] = parameter_B0;
    parameter_A[1] = parameter_A1;
    parameter_B[1] = parameter_B1;
    parameter_A[2] = parameter_A2;
    parameter_B[2] = parameter_B2;

	if( ix+offset_x0 >= 0 && iy+offset_y0 >= 0 && ix+offset_x0 < rsAllocationGetDimX(bitmap0) && iy+offset_y0 < rsAllocationGetDimY(bitmap0) ) {
    	pixels[0] = rsGetElementAt_uchar4(bitmap0, x+offset_x0, y+offset_y0);
	}
	else {
    	pixels[0] = in;
        parameter_A[0] = parameter_A[mid_indx];
        parameter_B[0] = parameter_B[mid_indx];
	}

    // middle image is not offset
    pixels[1] = in;

 	if( ix+offset_x2 >= 0 && iy+offset_y2 >= 0 && ix+offset_x2 < rsAllocationGetDimX(bitmap2) && iy+offset_y2 < rsAllocationGetDimY(bitmap2) ) {
    	pixels[2] = rsGetElementAt_uchar4(bitmap2, x+offset_x2, y+offset_y2);
	}
	else {
    	pixels[2] = in;
        parameter_A[2] = parameter_A[mid_indx];
        parameter_B[2] = parameter_B[mid_indx];
	}

	float3 hdr = (float3){0.0f, 0.0f, 0.0f};
	float sum_weight = 0.0f;

	// calculateHDR
	if( false )
	{
        for(int i=0;i<n_bitmaps;i++) {
            float3 rgb = convert_float3(pixels[i].rgb);
            /*if( pixels[i].r == 255 || pixels[i].g == 255 || pixels[i].b == 255 ) {
                // images should be ordered from dark to bright
                // if we reach a saturated pixel, then don't want any contribution from it; also we
                // assume that all brighter images have saturated pixels (if they don't, it must be noise)
                // if this is the first image, all pixels are saturated
                if( i == 0 ) {
                    // response function
                    rgb = parameter_A[1] * rgb + parameter_B[1];
                    hdr = rgb;
                    sum_weight = 1.0f;
                }
                break;
            }*/
            float avg = (rgb.r+rgb.g+rgb.b) / 3.0f;
            // weight_scale_c chosen so that 0 and 255 map to a non-zero weight of 1.0/127.5
            float weight = 1.0f - weight_scale_c * fabs( 127.5f - avg );
            /*const float safe_range_c = 96.0f;
            float diff = fabs( avg - 127.5f );
            float weight = 1.0f;
            if( diff > safe_range_c ) {
                // scaling chosen so that 0 and 255 map to a non-zero weight of 0.01
                weight = 1.0f - 0.99f * (diff - safe_range_c) / (127.5f - safe_range_c);
            }*/

            // response function
            rgb = parameter_A[i] * rgb + parameter_B[i];

            hdr += weight * rgb;
            sum_weight += weight;
        }
	}
	// assumes 3 bitmaps, with middle bitmap being the "base" exposure, and first image being darker, third image being brighter
	{
		//const float safe_range_c = 64.0f;
		const float safe_range_c = 96.0f;
        float3 rgb = convert_float3(pixels[mid_indx].rgb);
		float avg = (rgb.r+rgb.g+rgb.b) / 3.0f;
		float diff = fabs( avg - 127.5f );
		float weight = 1.0f;
		if( diff > safe_range_c ) {
			// scaling chosen so that 0 and 255 map to a non-zero weight of 0.01
			weight = 1.0f - 0.99f * (diff - safe_range_c) / (127.5f - safe_range_c);
		}

		// response function
		rgb = parameter_A[mid_indx] * rgb + parameter_B[mid_indx];

		hdr += weight * rgb;
		sum_weight += weight;

		if( weight < 1.0 ) {
    		float3 base_rgb = rgb;

			// now look at a neighbour image
			weight = 1.0f - weight;
			if( avg <= 127.5f ) {
                rgb = convert_float3(pixels[mid_indx+1].rgb);
    			/* In some cases it can be that even on the neighbour image, the brightness is too
    			   dark/bright - but it should still be a better choice than the base image.
    			   If we change this (including say for handling more than 3 images), need to be
    			   careful of unpredictable effects. In particular, consider a pixel that is brightness
    			   255 on the base image. As the brightness on the neighbour image increases, we
    			   should expect that the resultant image also increases (or at least, doesn't
    			   decrease). See testHDR36 for such an example.
    			   */
				/*avg = (rgb.r+rgb.g+rgb.b) / 3.0f;
				diff = fabs( avg - 127.5f );
				if( diff > safe_range_c ) {
					// scaling chosen so that 0 and 255 map to a non-zero weight of 0.01
					weight *= 1.0f - 0.99f * (diff - safe_range_c) / (127.5f - safe_range_c);
				}*/

				rgb = parameter_A[mid_indx+1] * rgb + parameter_B[mid_indx+1];
			}
			else {
                rgb = convert_float3(pixels[mid_indx-1].rgb);
				// see note above for why this is commented out
				/*avg = (rgb.r+rgb.g+rgb.b) / 3.0f;
				diff = fabs( avg - 127.5f );
				if( diff > safe_range_c ) {
					// scaling chosen so that 0 and 255 map to a non-zero weight of 0.01
					weight *= 1.0f - 0.99f * (diff - safe_range_c) / (127.5f - safe_range_c);
				}*/

				rgb = parameter_A[mid_indx-1] * rgb + parameter_B[mid_indx-1];
			}

            float value = fmax(rgb.r, rgb.g);
            value = fmax(value, rgb.b);
			if( value <= 250.0f )
			{
                // deghosting
                // for overexposed pixels, we don't have a reliable value for that pixel, so we can't distinguish between
                // pixels that are overexposed, and those that need deghosting, so we limit to value <= 250.0f
                // tests that benefit from deghosting for dark pixels: testHDR2, testHDR9, testHDR19, testHDR21, testHDR30,
                // testHDR35, testHDR37, testHDR40, testHDR41, testHDR42, testHDR44
                // tests that benefit from deghosting for bright pixels: testHDR2, testHDR41, testHDR42
                // for 127.5-avg = 96.0, we want wiener_C = wiener_C_lo
                // for 127.5-avg = 127.5f, we want wiener_C = wiener_C_hi
                const float wiener_C_lo = 2000.0f;
                const float wiener_C_hi = 8000.0f;
                float wiener_C = wiener_C_lo; // higher value means more HDR but less ghosting
                float x = fabs( value - 127.5f ) - 96.0f;
                if( x > 0.0f ) {
                    const float scale = (wiener_C_hi-wiener_C_lo)/(127.5f-96.0f);
                    wiener_C = wiener_C_lo + x*scale;
                }
                float3 diff = base_rgb - rgb;
                float L = dot(diff, diff);
                float ghost_weight = L/(L+wiener_C);
                rgb = ghost_weight * base_rgb + (1.0-ghost_weight) * rgb;
            }

			hdr += weight * rgb;
			sum_weight += weight;
			
			// testing: make all non-safe images black:
			//hdr_r = 0;
			//hdr_g = 0;
			//hdr_b = 0;
		}
	}

	hdr /= sum_weight;

    uchar4 out = tonemap(hdr);
	return out;
}

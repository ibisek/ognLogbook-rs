use log::{info, error};

use gdal::Dataset;
use gdal::spatial_ref::{SpatialRef, CoordTransform};
// use gdal::raster::RasterBand;

pub struct GeoFile { 
    xsize: i64,
    ysize: i64,
    dataset: Dataset,
    // band: RasterBand<'a>,
    geotransform: [f64; 6],
    ct: CoordTransform,
}

impl GeoFile {

    pub fn new(geotiff_filepath: &str) -> GeoFile {
        info!("Reading geotiff from '{geotiff_filepath}'");

        let dataset = Dataset::open(geotiff_filepath).unwrap();
        // println!("This {} is in '{}' and has {} bands.", dataset.driver().long_name(), dataset.spatial_ref().unwrap().name().unwrap(), dataset.raster_count());
        
        // let band = dataset.rasterband(1).unwrap();

        let (xsize, ysize) = dataset.raster_size();
        // println!("RASTER SIZE {xsize} X {ysize}");
    
        let geotransform = dataset.geo_transform().unwrap();
        
        let src_ref = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
        let dst_ref = dataset.spatial_ref().unwrap();
    
        // let sn = src_ref.name().unwrap();
        // let dn = dst_ref.name().unwrap();
        // println!("CT from {sn} to {dn}");

        let ct = CoordTransform::new(&src_ref, &dst_ref).unwrap();

        Self {
            xsize: xsize as i64,
            ysize: ysize as i64,
            dataset: dataset,
            // band: band,
            geotransform: geotransform,
            ct: ct,
        }
    }

    pub fn get_value(&mut self, lat:f64, lon: f64) -> Option<i64> {
        // transform coordinates between spatial refs (from [lat,lon] to [x,y]):
        let mut xs = [lat];
        let mut ys = [lon];
        let mut zs = [];
        match self.ct.transform_coords(&mut xs, &mut ys, &mut zs) {
            Ok(_) => (),
            Err(_) => {
                error!("Wrong argumens into transform_coords(): lat:'{:.4}'; lon:'{:.4}'", lat, lon);
                return None // wrongly parsed coords were passed?
            },
        }
        
        // let x = xs[0];
        // let y = ys[0];
        // println!("XY1: {x}; {y}");
        
        let xy = [xs[0], ys[0]];
    
        // calc xy coords pointing to the geotiff:
        let x = ((xy[0] - self.geotransform[0]) / self.geotransform[1]).round() as i64;
        let y = ((xy[1] - self.geotransform[3]) / self.geotransform[5]).round() as i64;
        // println!("XY2: {x}; {y}");

        // TODO if 0 <= x < self.dataset.RasterXSize and 0 <= y < self.dataset.RasterYSize:
        if x >= 0 && x < self.xsize && y >= 0 && y < self.ysize {
            // read out the value at the geotiff XY position:
            let buf = self.dataset.rasterband(1).unwrap().read_as::<f64>((x as isize,y as isize), (1,1), (1,1), Option::None).unwrap();
            let mut elevation = buf.data()[0].round() as i64;  // [m]
            
            if elevation < -10_994 || elevation > 100*1000 {    // below depth of Mariana Trench (10994m) or above space edge (100km)
                elevation = 0;
            }
            
            return Some(elevation);
        }
    
        return None;
        
    }

}

pub struct MeshOBJ {
    pub positions: Vec<[f32;3]>,
    pub normals:   Vec<[f32;3]>,
    pub uvs:       Vec<[f32;2]>,
    pub colors:    Vec<[f32;3]>,
    pub faces:     Vec<(u32, u32, u32)>,
}

impl MeshOBJ {
    pub fn new_empty() -> Self {
        Self {
            positions: Vec::new(),
            normals:   Vec::new(),
            uvs:       Vec::new(),
            colors:    Vec::new(),
            faces:     Vec::new(),
        }
    }

    pub fn as_opengl_format(&self) -> ( Vec<f32>, Vec<u32> ) {

        let mut vertices:     Vec<f32> = Vec::with_capacity( self.faces.len() );
        let mut mesh_indeces: Vec<u32> = Vec::with_capacity( self.faces.len() );

        for( idx, index ) in self.faces.iter().enumerate() {

            let pos_i    = (index.0 - 1) as usize;
            let uv_i     = (index.1 - 1) as usize;
            let normal_i = (index.2 - 1) as usize;

            vertices.push( self.positions[pos_i][0].clone() );
            vertices.push( self.positions[pos_i][1].clone() );
            vertices.push( self.positions[pos_i][2].clone() );

            vertices.push( self.normals[normal_i][0].clone() );
            vertices.push( self.normals[normal_i][1].clone() );
            vertices.push( self.normals[normal_i][2].clone() );

            vertices.push( self.uvs[uv_i][0].clone() );
            vertices.push( self.uvs[uv_i][1].clone() );

            mesh_indeces.push( idx as u32 );

        }

        ( vertices, mesh_indeces )
    }
}

impl core::fmt::Display for MeshOBJ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let msg2 = |vec:&Vec<[f32;2]>| -> String {
            let mut buffer = String::new();
            for p in vec.iter() {
                buffer.push_str(
                    &format!(
                        "vt {:7.3} {:7.3} \n",
                        p[0], p[1]
                    )
                )
            }
            buffer
        };

        let msg3 = |vec:&Vec<[f32;3]>, kind:&str| -> String {
            let mut buffer = String::new();
            for p in vec.iter() {
                buffer.push_str(
                    &format!(
                        "{} {:7.3} {:7.3} {:7.3}\n",
                        kind, p[0], p[1], p[2]
                    )
                )
            }
            buffer
        };

        let msg_u32 = |vec:&Vec<(u32, u32, u32)>| -> String {
            let mut buffer = String::from( "f " );
            for ( idx, p ) in vec.iter().enumerate() {
                buffer.push_str( &format!("{:6}/{:6}/{:6} ", p.0, p.1, p.2 ) );
                if idx % 3 == 0 {
                    buffer.push_str( &format!( "\nf " ) );
                }
            }
            buffer
        };

        let colors_msg = if self.colors.is_empty() { "".to_owned() }
        else { msg3( &self.colors, "color" ) };

        write!(
            f, "{}{}{}{}{}",
            msg3( &self.positions, "v",  ),
            msg2( &self.uvs ),
            msg3( &self.normals,   "vn", ),
            colors_msg,
            msg_u32( &self.faces ),
        )
    }
}

pub fn parse_obj( src:String ) -> Result<Vec<MeshOBJ>, Error> {

    let parser =
    | lines:Vec<&str>, index_offset:( u32, u32, u32 ) | -> Result<( MeshOBJ, ( u32, u32, u32 ) ), Error> {
        let mut mesh = MeshOBJ::new_empty();
        let mut next_index_offset = ( 0u32, 0u32, 0u32 );
        // parse data out of text
        // iterate through each line
        for line in lines.iter() {
            // skip empty lines
            if line.is_empty() { continue; }
            // skip comment lines
            if line.contains( COMMENT ) { continue; }
    
            // get symbols
            let symbols:Vec<&str> = line.split_whitespace().collect();
    
            let parse_floats = | symbols:Vec<&str> | -> Result<Vec<f32>, Error> {
                let mut result = Vec::new();
                for symbol in symbols.iter().skip(1) {
    
                    let data = match symbol.parse::<f32>() {
                        Ok(f) => f,
                        Err(e) => return Err(
                            Error::ParseFloat( format!("Parse Float Error: {}", e) )
                        ),
                    };
    
                    result.push( data );
                }
                Ok( result )
            };
    
            // only parse if first symbol is recognized
            match symbols[0] {
                POSITION => {
                    let result = parse_floats( symbols )?;
                    match result.len() {
                        3 => { // position
                            mesh.positions.push( [ result[0], result[1], result[2] ] );
                        },
                        6 => { // position + color
                            mesh.positions.push( [ result[0], result[1], result[2] ] );
                            mesh.colors.push(    [ result[3], result[4], result[5] ] );
                        }
                        _ => { // error
                            return Err( Error::PositionColorFormat );
                        }
                    }
                    
                }
                UV       => {
                    let result = parse_floats( symbols )?;
                    match result.len() {
                        2 => {
                            mesh.uvs.push( [ result[0], result[1] ] );
                        }
                        _ => {
                            return Err( Error::UVFormat );
                        }
                    }
                }
                NORMAL   => {
                    let result = parse_floats( symbols )?;
                    match result.len() {
                        3 => {
                            mesh.normals.push( [ result[0], result[1], result[2] ] );
                        }
                        _ => {
                            return Err( Error::NormalFormat );
                        }
                    }
                }
                INDEX    => {
                    for symbol in symbols.iter().skip(1) {
                        let mut result:Vec<u32> = Vec::new();
                        let sub_symbols:Vec<&str> = symbol.split('/').collect();
    
                        for sub_symbol in sub_symbols {
                            let data = match sub_symbol.parse::<u32>() {
                                Ok(u) => u,
                                Err(e) => return Err(
                                    Error::ParseInt( format!( "Parse Int Error: {}", e ) )
                                ),
                            };
                            result.push( data );
                        }
    
                        match result.len() {
                            3 => {
                                if result[0] > next_index_offset.0 { next_index_offset.0 = result[0] }
                                if result[1] > next_index_offset.1 { next_index_offset.1 = result[1] }
                                if result[2] > next_index_offset.2 { next_index_offset.2 = result[2] }
                                mesh.faces.push(
                                    (
                                        result[0] - index_offset.0,
                                        result[1] - index_offset.1,
                                        result[2] - index_offset.2
                                    )
                                );
                            },
                            _ => return Err( Error::FaceIndexFormat ),
                        }
    
                    }
                }
                _ => { continue; }
            };
    
        }
        Ok(( mesh, next_index_offset ))
    };

    let mut object_buffer:Vec<MeshOBJ> = Vec::new();
    // face indeces keep going up, they don't reset to 1 when in a new object -_-
    // offset is used to correct indeces when reading MeshOBJ
    let mut last_index_offset = ( 0, 0, 0 );

    let objects_raw:Vec<&str> = src.split( "o " ).collect();
    // skip first, just header comments
    for object_raw in objects_raw.iter().skip(1) {
        let lines:Vec<&str> = object_raw.split( '\n' ).collect();
        let ( obj, next_index_offset ) = parser( lines, last_index_offset )?;
        last_index_offset = next_index_offset;
        object_buffer.push( obj );
    }

    Ok( object_buffer )
    
}

#[derive(Debug)]
pub enum Error {
    ParseFloat(String),
    ParseInt(String),
    PositionColorFormat,
    UVFormat,
    NormalFormat,
    FaceIndexFormat,
}

impl Error {
    pub fn msg(&self) -> String {
        match self {
            Error::ParseFloat(s) => s.clone(),
            Error::ParseInt(s)   => s.clone(),
            Error::PositionColorFormat    =>
                format!("Formatting Error: Positions/Vertex Colors are not properly formatted!"),
            Error::UVFormat =>
                format!("Formatting Error: UVs are not properly formatted!"),
            Error::NormalFormat => 
                format!("Formatting Error: Normals are not properly formatted!"),
            Error::FaceIndexFormat => 
                format!("Formatting Error: Face Indeces are not properly formatted!"),
                
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!( f, "{}", self.msg() )
    }
}

const COMMENT:char  = '#';
const POSITION:&str = "v";
const UV:&str       = "vt";
const NORMAL:&str   = "vn";
const INDEX:&str    = "f";

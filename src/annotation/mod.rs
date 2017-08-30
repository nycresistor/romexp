pub trait Annotation {
    fn span(&self) -> (usize, usize);
    fn comments(&self) -> &str;
    fn type_str(&self) -> &str;
}

pub trait AnnotationEngine {
    fn new() -> Self;
    fn build_annotations(&self, raw_data : &[u8]) -> Vec<Box<Annotation>>;
}

pub struct CStringAnnotation {
    start : usize,
    end : usize,
    contents : String,
}

impl Annotation for CStringAnnotation {
    fn span(&self) -> (usize, usize) { (self.start, self.end) }
    fn comments(&self) -> &str { self.contents.as_str() }
    fn type_str(&self) -> &str { "ASCII String" }
}

pub struct CStringAnnotationEngine { }

static ASCII_LOOKUP : [bool;128] = [
    false, false, false, false,    false, false, false, false,
    false, true,  true,  false,    false, true,  false, false,
    false, false, false, false,    false, false, false, false,
    false, false, false, false,    false, false, false, false,

    true,  true,  true,  true,     true,  true,  true,  true,
    true,  true,  true,  true,     true,  true,  true,  true,
    true,  true,  true,  true,     true,  true,  true,  true,
    true,  true,  true,  true,     true,  true,  true,  true,

    true,  true,  true,  true,     true,  true,  true,  true,
    true,  true,  true,  true,     true,  true,  true,  true,
    true,  true,  true,  true,     true,  true,  true,  true,
    true,  true,  true,  true,     true,  true,  true,  true,

    true,  true,  true,  true,     true,  true,  true,  true,
    true,  true,  true,  true,     true,  true,  true,  true,
    true,  true,  true,  true,     true,  true,  true,  true,
    true,  true,  true,  true,     true,  true,  true,  false, ];

    
impl AnnotationEngine for CStringAnnotationEngine {
    fn new() -> Self {
        CStringAnnotationEngine {}
    }

    fn build_annotations(&self, raw_data : &[u8]) -> Vec<Box<Annotation>> {
        let mut annotations : Vec<Box<Annotation>> = Vec::new();
        // Iterate through and find null-terminated sequences of ascii
        // characters longer than 4 chars
        let min_str_len = 4;

        let mut start : Option<usize> = None;
        let mut idx = 0;
        while idx < raw_data.len() {
            let c = raw_data[idx];
            if c == 0 {
                match start {
                    None => (),
                    Some(w) if idx-w > min_str_len => {
                        let v : Vec<u8> = raw_data[w..idx+1].to_vec();
                        let a = CStringAnnotation
                        { start : w,
                          end : idx+1,
                         contents : String::from_utf8(v).unwrap() };
                        start = None;
                        annotations.push(Box::new(a));
                    },
                    _ => { start = None; }
                }
            } else if (c >= 128) || !ASCII_LOOKUP[c as usize] {
                start = None;
            } else if start == None {
                start = Some(idx);
            }
            idx = idx + 1;
        }
        annotations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn ascii_annotation_engine() {
        static ASCII_TEST: &'static [u8] = include_bytes!("../../sample_binaries/strings-test.bin");
        let engine = CStringAnnotationEngine::new();
        let annotations = engine.build_annotations(ASCII_TEST);
        assert_eq!(3,annotations.len());
        
    }
}

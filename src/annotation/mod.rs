pub trait Annotation {
    fn span(&self) -> (usize, usize);
    fn comments(&self) -> &str;
    fn type_str(&self) -> &str;
}

pub trait AnnotationEngine {
    fn new() -> Self;
    fn build_annotations(&self, raw_data : &u8) -> Vec<Box<Annotation>>;
}

struct AsciiStringAnnotation {
    start : usize,
    end : usize,
    contents : String,
}

impl Annotation for AsciiStringAnnotation {
    fn span(&self) -> (usize, usize) { (self.start, self.end) }
    fn comments(&self) -> &str { self.contents.as_str() }
    fn type_str(&self) -> &str { "ASCII String" }
}

struct AsciiStringAnnotationEngine { }

impl AnnotationEngine for AsciiStringAnnotationEngine {
    fn new() -> Self {
        AsciiStringAnnotationEngine {}
    }

    fn build_annotations(&self, raw_data : &u8) -> Vec<Box<Annotation>> {
        let mut annotations = Vec::new();
        
        annotations
    }
}

    
    
    

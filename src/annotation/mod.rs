pub trait Annotation {
    fn span(&self) -> (usize, usize);
    fn comments(&self) -> &str;
    fn type_str(&self) -> &str;
    fn confidence(&self) -> u8;
}

pub trait AnnotationEngine {
    fn new() -> Self;
    fn build_annotations(&self, raw_data : &[u8]) -> AnnotationStore;
}

pub struct AnnotationStore {
    v : Vec<Box<Annotation>>,
    title : String,
}

use std::slice::Iter;

impl AnnotationStore {
    pub fn new(title : &str) -> AnnotationStore {
        AnnotationStore { v : Vec::new(), title : String::from(title), }
    }
    pub fn insert(&mut self, a : Box<Annotation>) {
        self.v.push(a)
    }

    pub fn query<'a>(&'a self, point : usize) -> Vec<&'a Box<Annotation>> {
        let mut v = Vec::new();
        for a in &self.v {
            let span = a.span();
            if (point >= span.0) && (point <= span.1) {
                v.push(a);
            }
        }
        v
    }

    pub fn iter(&self) -> Iter<Box<Annotation>> { self.v.iter() }
}

impl IntoIterator for AnnotationStore {
    type Item = Box<Annotation>;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter { self.v.into_iter() }
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
    fn confidence(&self) -> u8 { 255 }
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

    fn build_annotations(&self, raw_data : &[u8]) -> AnnotationStore {
        let mut annotations = AnnotationStore::new("C Strings");
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
                        let v : Vec<u8> = raw_data[w..idx].to_vec();
                        let a = CStringAnnotation
                        { start : w,
                          end : idx+1,
                         contents : String::from_utf8(v).unwrap() };
                        start = None;
                        annotations.insert(Box::new(a));
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
        assert_eq!(3,annotations.v.len());
        
    }

    #[test]
    fn annotation_store() {
        let mut store = AnnotationStore::new("test store");
        fn mka( start : usize, end : usize ) -> Box<Annotation> {
            let c = format!("{}-{}",start,end).to_string();
            Box::new(CStringAnnotation { start : start, end : end, contents : c })
        }
        store.insert(mka(1,20));
        store.insert(mka(5,10));
        store.insert(mka(9,30));
        assert_eq!(0,store.query(0).len());
        assert_eq!(1,store.query(1).len());
        assert_eq!(2,store.query(6).len());
        assert_eq!(3,store.query(9).len());
    }
}


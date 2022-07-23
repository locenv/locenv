pub fn parse<'pattern>(pattern: &'pattern str) -> Result<Vec<Segment<'pattern>>, ParseError> {
    let mut segments: Vec<Segment<'pattern>> = Vec::new();

    for segment in pattern.split_terminator('/').skip(1) {
        // Disallow consecutive '/'.
        if segment.is_empty() {
            return Err(ParseError::InvalidPattern);
        }

        // Check if segment is a parameter.
        let is_param = if let Some(i) = segment.find('{') {
            // Disallow '{' in the middle.
            if i != 0 {
                return Err(ParseError::InvalidPattern);
            }

            if let Some(i) = segment.rfind('}') {
                if i != (segment.len() - 1) {
                    // '}' must be on the end.
                    return Err(ParseError::InvalidPattern);
                }

                true
            } else {
                // No '}' in the segment.
                return Err(ParseError::InvalidPattern);
            }
        } else {
            false
        };

        // Create segment data.
        segments.push(if is_param {
            let options = &segment[1..(segment.len() - 1)];

            if options.is_empty() {
                Segment::Param
            } else {
                return Err(ParseError::InvalidPattern);
            }
        } else {
            Segment::Static(segment)
        });
    }

    Ok(segments)
}

pub enum Segment<'pattern> {
    Static(&'pattern str),
    Param,
}

pub enum ParseError {
    InvalidPattern,
}

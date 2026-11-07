# AI-assisted development and clean-room practice

MatRust Studio has been developed with AI assistance. AI tools may help
explain code, suggest designs, draft tests, or review changes. Their output is
not treated as authoritative: the maintainer is responsible for checking
behavior, source provenance, licenses, and tests before accepting a change.

This project follows a clean-room policy for MATLAB compatibility:

- Work from mathematical definitions, public file-format specifications,
  public documentation, and permitted black-box observations.
- Do not use proprietary MathWorks source, binaries, P-code, or license
  mechanisms as inputs to implementation or AI prompts.
- Do not copy or adapt GNU Octave or other GPL-licensed implementation code
  or text. When learning from a source, independently implement the
  underlying idea and record the source and license in the change.
- Use synthetic fixtures created for the project. Do not commit real
  proprietary `.mlapp` or `.slx` files or private customer data.
- Review generated code line by line. Run the relevant tests and explain the
  source of compatibility decisions in the pull request.

These are project policies and review practices, not a legal certification or
a claim that no copied material exists anywhere in the repository's history.
Report a suspected provenance or licensing issue privately to the maintainer
before reusing or distributing the affected material.

AI tools must not receive secrets, private real-app files, personal data, or
licensed source material. Contributors should disclose substantial
AI-generated contributions in the pull request and remain accountable for
them.

// code from https://codepen.io/dVinci/pen/WyOJBN, modified
class Slider {
    constructor($fir = document.getElementsByClassName('fir')[0]) {
        this.dom = {
            $fir: $fir,
            $range: $fir.querySelector('.fir-range'),
            $counter: document.createElement('div'),
            $line: document.createElement('div')
        };
        this.dimension = {
            min: Math.floor(this.dom.$range.getAttribute('min')),
            max: Math.floor(this.dom.$range.getAttribute('max')),
        };
        this.rangeWidth = this.dom.$range.offsetWidth;
        this.position = 0;

        this.init();
    }

    init() {
        this.initHover();
        this.initRange();
        this.initCounter();
        this.initLine();
        this.initAnimate();
    }

    initHover() {
        let leaveTimer;
        this.dom.$fir.addEventListener('mouseenter', () => {
            clearTimeout(leaveTimer);
            this.dom.$counter.classList.add('hover');
        });
        this.dom.$fir.addEventListener('mouseleave', () => {
            leaveTimer = setTimeout(() => {
                this.dom.$counter.classList.remove('hover');
            }, 500);
        });
    }

    initRange() {
        this.textSuperPosition = this.dom.$range.value;
        this.changeVals();

        this.dom.$range.addEventListener('input', this.changeVals.bind(this));
    }

    initCounter() {
        this.changeCounter();

        this.dom.$counter.setAttribute('class', 'fir-counter');
        this.dom.$fir.appendChild(this.dom.$counter);
    }

    initLine() {
        this.changeCounter();

        this.dom.$line.setAttribute('class', 'fir-line');
        this.dom.$fir.appendChild(this.dom.$line);
    }

    initAnimate() {
        setInterval(() => requestAnimationFrame(this.changeCounter.bind(this)), 1000 / 60);
    }

    changeVals() {
        this.amount = Math.floor(this.dom.$range.value);
        this.range = (this.amount - this.dimension.min) / (this.dimension.max - this.dimension.min);
    }

    changeCounter() {
        this.textSuperPosition = this.lerp(this.textSuperPosition, this.amount, 0.1);

        this.position = this.lerp(this.position, this.rangeWidth * this.range, 0.1);
        let newSize = this.lerp(this.dom.$line.style.getPropertyValue('--size'), this.range, 0.1);
        let logicCounter = this.amount - this.textSuperPosition

        this.dom.$counter.textContent = Math.floor((logicCounter > 0 && logicCounter < 1 ? this.amount : this.textSuperPosition));
        this.dom.$counter.style.setProperty('--position', `${this.position}px`);
        this.dom.$line.style.setProperty('--size', newSize);
    }

    lerp(start, end, amt) {
        return (1 - amt) * start + amt * end;
    }

    setValue(value) {
        this.dom.$range.value = value;
        this.changeVals();
    }
}